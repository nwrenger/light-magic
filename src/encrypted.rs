use aes::Aes256;
use argon2::{self, Argon2, Params};
use ctr::cipher::{KeyIvInit, StreamCipher};
use hmac::{Hmac, Mac};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use rand::{rngs::OsRng, RngCore};
use rmp_serde::{decode, encode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::Sha256;
use std::{
    ffi::{OsStr, OsString},
    fmt,
    fs::{self, File},
    io::{self, Read, Write},
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};
use tracing::{error, info};
use zeroize::Zeroize;

// Type definitions
type Aes256Ctr = ctr::Ctr128BE<Aes256>;
type HmacSha256 = Hmac<Sha256>;

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 16; // 128-bit nonce for AES-CTR

/// Structure to hold encrypted data along with salt, nonce, and HMAC
#[derive(Serialize, Deserialize)]
pub struct EncryptedData {
    salt: Vec<u8>,
    nonce: Vec<u8>,
    ciphertext: Vec<u8>,
    hmac: Vec<u8>,
}

/// This trait needs to be implemented for the Database struct.
pub trait EncryptedDataStore: Default + Serialize {
    /// Opens a Database by the specified path and password.
    fn open<P>(db: P, password: &str) -> io::Result<EncryptedAtomicDatabase<Self>>
    where
        P: AsRef<Path>,
        Self: DeserializeOwned,
    {
        let db_path = db.as_ref();
        if db_path.exists() {
            EncryptedAtomicDatabase::load(db_path, password)
        } else {
            EncryptedAtomicDatabase::create_new(db_path, password)
        }
    }

    /// Load the database from a string with the provided password and save it to the filesystem.
    fn create_from_str<P>(
        data: &str,
        path: P,
        password: &str,
    ) -> io::Result<EncryptedAtomicDatabase<Self>>
    where
        P: AsRef<Path>,
        Self: DeserializeOwned,
    {
        let db_path = path.as_ref();
        if !db_path.exists() {
            EncryptedAtomicDatabase::create_from_str(data, path, password)
        } else {
            Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "A file already exists at the provided path!",
            ))
        }
    }

    /// Loads file data into the `Database` after decrypting it.
    fn load_encrypted(file: impl Read, key: &[u8]) -> io::Result<Self>
    where
        Self: DeserializeOwned,
    {
        let encrypted: EncryptedData = decode::from_read(file).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to deserialize encrypted data: {}", e),
            )
        })?;

        Self::decrypt(&encrypted, key)
    }

    /// Saves data of the `Database` to a file after encrypting it.
    fn save_encrypted(&self, mut file: impl Write, key: &[u8], salt: &[u8]) -> io::Result<()> {
        let encrypted = self.encrypt(key, salt)?;
        encode::write(&mut file, &encrypted).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to write encrypted data to file: {}", e),
            )
        })
    }

    /// Encrypts the current data and returns the encrypted data.
    fn encrypt(&self, key: &[u8], salt: &[u8]) -> io::Result<EncryptedData> {
        let mut nonce_bytes = vec![0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);

        let plaintext = encode::to_vec(self).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Serialization failed: {}", e),
            )
        })?;

        // Initialize cipher
        let mut cipher = Aes256Ctr::new(key.into(), nonce_bytes.as_slice().into());

        // Encrypt the plaintext in-place
        let mut ciphertext = plaintext.clone();
        cipher.apply_keystream(&mut ciphertext);

        // Compute HMAC
        let mut mac = HmacSha256::new_from_slice(key)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "HMAC initialization failed"))?;
        mac.update(&ciphertext);
        let hmac_bytes = mac.finalize().into_bytes().to_vec();

        Ok(EncryptedData {
            salt: salt.to_vec(),
            nonce: nonce_bytes,
            ciphertext,
            hmac: hmac_bytes,
        })
    }

    /// Decrypts the encrypted data using the given key and returns the decrypted data.
    fn decrypt(encrypted: &EncryptedData, key: &[u8]) -> io::Result<Self>
    where
        Self: DeserializeOwned,
    {
        // Verify HMAC
        let mut mac = HmacSha256::new_from_slice(key)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "HMAC initialization failed"))?;
        mac.update(&encrypted.ciphertext);
        mac.verify_slice(&encrypted.hmac).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "HMAC verification failed: Data is corrupted or tampered",
            )
        })?;

        // Initialize cipher
        let mut cipher = Aes256Ctr::new(key.into(), encrypted.nonce.as_slice().into());

        // Decrypt the ciphertext in-place
        let mut decrypted_bytes = encrypted.ciphertext.clone();
        cipher.apply_keystream(&mut decrypted_bytes);

        let data = decode::from_slice(&decrypted_bytes).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to deserialize decrypted data: {}", e),
            )
        })?;

        Ok(data)
    }
}

/// Derive a 32-byte key from the password and salt using Argon2id
fn derive_key(password: &str, salt: &[u8]) -> io::Result<[u8; 32]> {
    let params = Params::new(65536, 3, 1, None)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Invalid Argon2 parameters"))?;

    let mut key = [0u8; 32];
    Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params)
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Key derivation failed"))?;

    Ok(key)
}

/// Synchronized Wrapper that automatically saves changes when path and tmp are defined
pub struct EncryptedAtomicDatabase<T: EncryptedDataStore> {
    path: PathBuf,
    tmp: PathBuf,
    data: RwLock<T>,
    key: RwLock<[u8; 32]>,
    salt: RwLock<Vec<u8>>,
}

impl<T: EncryptedDataStore + DeserializeOwned> EncryptedAtomicDatabase<T> {
    /// Load the database from the file system with the provided password
    pub fn load<P: AsRef<Path>>(path: P, password: &str) -> io::Result<Self> {
        let new_path = path.as_ref().to_path_buf();
        let tmp = Self::tmp_path(&new_path)?;

        let file = File::open(&new_path)?;
        // First, deserialize to get the salt
        let encrypted: EncryptedData = decode::from_read(&file).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to deserialize encrypted data: {}", e),
            )
        })?;
        let key = derive_key(password, &encrypted.salt)?;

        // Re-open the file to reset the cursor
        let file = File::open(&new_path)?;
        let data = T::load_encrypted(file, &key)?;

        // Store the salt and key
        Ok(Self {
            path: new_path,
            tmp,
            data: RwLock::new(data),
            key: RwLock::new(key),
            salt: RwLock::new(encrypted.salt),
        })
    }

    /// Load the database from a string with the provided password and save it to the filesystem.
    /// It checks if the provided password can decrypt the content successfully before saving it.
    pub fn create_from_str<P: AsRef<Path>>(
        data: &str,
        path: P,
        password: &str,
    ) -> io::Result<Self> {
        let new_path = path.as_ref().to_path_buf();
        let tmp = Self::tmp_path(&new_path)?;

        let encrypted: EncryptedData = decode::from_slice(data.as_bytes()).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to deserialize encrypted data: {}", e),
            )
        })?;

        let key = derive_key(password, &encrypted.salt)?;

        let data = T::decrypt(&encrypted, &key)?;
        atomic_write_encrypted(&tmp, &new_path, &data, &key, &encrypted.salt)?;

        Ok(Self {
            path: new_path,
            tmp,
            data: RwLock::new(data),
            key: RwLock::new(key),
            salt: RwLock::new(encrypted.salt),
        })
    }

    /// Create a new database and save it with the provided password.
    pub fn create_new<P: AsRef<Path>>(path: P, password: &str) -> io::Result<Self> {
        let new_path = path.as_ref().to_path_buf();
        let tmp = Self::tmp_path(&new_path)?;

        // Generate a fixed salt for the database
        let mut salt = vec![0u8; SALT_LEN];
        OsRng.fill_bytes(&mut salt);
        let key = derive_key(password, &salt)?;

        let data = Default::default();
        atomic_write_encrypted(&tmp, &new_path, &data, &key, &salt)?;

        Ok(Self {
            path: new_path,
            tmp,
            data: RwLock::new(data),
            key: RwLock::new(key),
            salt: RwLock::new(salt),
        })
    }

    /// Lock the database for reading.
    pub fn read(&self) -> EncryptedAtomicDatabaseRead<'_, T> {
        EncryptedAtomicDatabaseRead {
            data: self.data.read(),
        }
    }

    /// Lock the database for writing. This will save the changes atomically on drop.
    pub fn write(&self) -> EncryptedAtomicDatabaseWrite<'_, T> {
        // Clone the current key and salt references
        let key = *self.key.read();
        let salt = self.salt.read().clone();

        EncryptedAtomicDatabaseWrite {
            path: self.path.as_ref(),
            tmp: self.tmp.as_ref(),
            data: self.data.write(),
            key,
            salt,
        }
    }

    /// Change the password of the database. This will re-encrypt the data with a new key derived from the new password.
    pub fn change_password(&self, new_password: &str) -> io::Result<()> {
        let data_guard = self.data.read();

        let mut new_salt = vec![0u8; SALT_LEN];
        OsRng.fill_bytes(&mut new_salt);

        let mut new_key = derive_key(new_password, &new_salt)?;

        atomic_write_encrypted(&self.tmp, &self.path, &*data_guard, &new_key, &new_salt)?;

        {
            let mut key_lock = self.key.write();
            key_lock.copy_from_slice(&new_key);
        }
        {
            let mut salt_lock = self.salt.write();
            *salt_lock = new_salt.clone();
        }

        // zeroize local ephemeral buffers
        new_key.zeroize();
        new_salt.fill(0);

        Ok(())
    }

    fn tmp_path(path: &Path) -> io::Result<PathBuf> {
        let mut tmp_name = OsString::from(".");
        tmp_name.push(path.file_name().unwrap_or(OsStr::new("db")));
        tmp_name.push("~");
        let tmp = path.with_file_name(tmp_name);
        if tmp.exists() {
            error!(
                "Found orphaned database temporary file '{tmp:?}'. The server has recently crashed or is already running. Delete this before continuing!"
            );
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Orphaned temporary file exists",
            ));
        }
        Ok(tmp)
    }
}

/// Atomic write routine with encryption
fn atomic_write_encrypted<T: EncryptedDataStore>(
    tmp: &Path,
    path: &Path,
    data: &T,
    key: &[u8],
    salt: &[u8],
) -> io::Result<()> {
    {
        let tmpfile = File::create(tmp)?;
        data.save_encrypted(tmpfile, key, salt)?;
    }
    fs::rename(tmp, path)?;
    Ok(())
}

impl<T: EncryptedDataStore> fmt::Debug for EncryptedAtomicDatabase<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EncryptedAtomicDatabase")
            .field("file", &self.path)
            .finish()
    }
}

impl<T: EncryptedDataStore> Drop for EncryptedAtomicDatabase<T> {
    fn drop(&mut self) {
        info!("Saving database");
        let data_guard = self.data.read();
        let key = *self.key.read();
        let salt = self.salt.read().clone();
        if let Err(e) = atomic_write_encrypted(&self.tmp, &self.path, &*data_guard, &key, &salt) {
            error!("Failed to save database: {}", e);
        }
    }
}

pub struct EncryptedAtomicDatabaseRead<'a, T: EncryptedDataStore> {
    data: RwLockReadGuard<'a, T>,
}

impl<'a, T: EncryptedDataStore> Deref for EncryptedAtomicDatabaseRead<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

pub struct EncryptedAtomicDatabaseWrite<'a, T: EncryptedDataStore> {
    tmp: &'a Path,
    path: &'a Path,
    data: RwLockWriteGuard<'a, T>,
    key: [u8; 32],
    salt: Vec<u8>,
}

impl<'a, T: EncryptedDataStore> Deref for EncryptedAtomicDatabaseWrite<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a, T: EncryptedDataStore> DerefMut for EncryptedAtomicDatabaseWrite<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<'a, T: EncryptedDataStore> Drop for EncryptedAtomicDatabaseWrite<'a, T> {
    fn drop(&mut self) {
        info!("Saving database");
        if let Err(e) =
            atomic_write_encrypted(self.tmp, self.path, &*self.data, &self.key, &self.salt)
        {
            error!("Failed to save database: {}", e);
        }
    }
}
