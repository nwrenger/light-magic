use aes_gcm::{
    aead::{rand_core::RngCore, Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{self, Argon2};
use bincode::{self, Options};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
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

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;

#[inline]
fn bincode_cfg() -> impl bincode::Options {
    bincode::DefaultOptions::new().with_fixint_encoding()
}

/// Encrypted envelope (serialized with bincode)
#[derive(Serialize, Deserialize)]
pub struct EncryptedData {
    salt: [u8; SALT_LEN],
    nonce: [u8; NONCE_LEN],
    ciphertext: Vec<u8>,
}

/// Implement on your Database type.
pub trait EncryptedDataStore: Default + Serialize {
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

    /// Loads after decrypting `EncryptedData` read from `file`.
    fn load_encrypted(file: impl Read, key: &Key<Aes256Gcm>) -> io::Result<Self>
    where
        Self: DeserializeOwned,
    {
        let encrypted: EncryptedData = bincode_cfg().deserialize_from(file).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to deserialize encrypted data: {e}"),
            )
        })?;
        Self::decrypt(&encrypted, key)
    }

    /// Saves current state as encrypted `EncryptedData` to `file`.
    fn save_encrypted(
        &self,
        mut file: impl Write,
        key: &Key<Aes256Gcm>,
        salt: [u8; SALT_LEN],
    ) -> io::Result<()> {
        let encrypted = self.encrypt(key, salt)?;
        bincode_cfg()
            .serialize_into(&mut file, &encrypted)
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to write encrypted data to file: {e}"),
                )
            })
    }

    /// Encrypts the current data and returns the envelope.
    fn encrypt(&self, key: &Key<Aes256Gcm>, salt: [u8; SALT_LEN]) -> io::Result<EncryptedData> {
        // Non-allocating nonce
        let mut nonce = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce);

        // Serialize plaintext
        let plaintext = bincode_cfg().serialize(self).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Serialization failed: {e}"),
            )
        })?;

        let cipher = Aes256Gcm::new(key);
        let ct = cipher
            .encrypt(Nonce::from_slice(&nonce), plaintext.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Encryption failed: {e}")))?;

        Ok(EncryptedData {
            salt,
            nonce,
            ciphertext: ct,
        })
    }

    /// Decrypts the envelope into data.
    fn decrypt(encrypted: &EncryptedData, key: &Key<Aes256Gcm>) -> io::Result<Self>
    where
        Self: DeserializeOwned,
    {
        let cipher = Aes256Gcm::new(key);
        let pt = cipher
            .decrypt(
                Nonce::from_slice(&encrypted.nonce),
                encrypted.ciphertext.as_ref(),
            )
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Decryption failed: Incorrect password or corrupted data. {e}"),
                )
            })?;

        let data = bincode_cfg().deserialize(&pt).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to deserialize decrypted data: {e}"),
            )
        })?;

        Ok(data)
    }
}

/// Derive a 32-byte key from the password and salt using Argon2id
fn derive_key(password: &str, salt: &[u8]) -> io::Result<Key<Aes256Gcm>> {
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Key derivation failed"))?;

    let out = *Key::<Aes256Gcm>::from_slice(&key);
    key.zeroize(); // wipe stack buffer
    Ok(out)
}

/// Synchronized Wrapper, that automatically saves changes when path and tmp are defined
pub struct EncryptedAtomicDatabase<T: EncryptedDataStore> {
    path: PathBuf,
    tmp: PathBuf,
    data: RwLock<T>,
    key: RwLock<Key<Aes256Gcm>>,
    salt: RwLock<[u8; SALT_LEN]>,
}

impl<T: EncryptedDataStore + DeserializeOwned> EncryptedAtomicDatabase<T> {
    /// Load the database with the provided password.
    pub fn load<P: AsRef<Path>>(path: P, password: &str) -> io::Result<Self> {
        let new_path = path.as_ref().to_path_buf();
        let tmp = Self::tmp_path(&new_path)?;

        // Read the whole envelope once; don't reopen
        let mut file = File::open(&new_path)?;
        let encrypted: EncryptedData = bincode_cfg().deserialize_from(&mut file).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to deserialize encrypted data: {e}"),
            )
        })?;
        let key = derive_key(password, &encrypted.salt)?;
        let data = T::decrypt(&encrypted, &key)?;

        Ok(Self {
            path: new_path,
            tmp,
            data: RwLock::new(data),
            key: RwLock::new(key),
            salt: RwLock::new(encrypted.salt),
        })
    }

    /// Load from an in-memory string and persist if successful.
    pub fn create_from_str<P: AsRef<Path>>(
        data: &str,
        path: P,
        password: &str,
    ) -> io::Result<Self> {
        let new_path = path.as_ref().to_path_buf();
        let tmp = Self::tmp_path(&new_path)?;

        let encrypted: EncryptedData = bincode_cfg().deserialize(data.as_bytes()).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to deserialize encrypted data: {e}"),
            )
        })?;
        let key = derive_key(password, &encrypted.salt)?;
        let data = T::decrypt(&encrypted, &key)?;

        atomic_write_encrypted(&tmp, &new_path, &data, &key, encrypted.salt)?;

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

        // Generate salt
        let mut salt_bytes = [0u8; SALT_LEN];
        OsRng.fill_bytes(&mut salt_bytes);
        let key = derive_key(password, &salt_bytes)?;

        let data = Default::default();
        atomic_write_encrypted(&tmp, &new_path, &data, &key, salt_bytes)?;

        Ok(Self {
            path: new_path,
            tmp,
            data: RwLock::new(data),
            key: RwLock::new(key),
            salt: RwLock::new(salt_bytes),
        })
    }

    /// Lock the database for reading.
    pub fn read(&self) -> EncryptedAtomicDatabaseRead<'_, T> {
        EncryptedAtomicDatabaseRead {
            data: self.data.read(),
        }
    }

    /// Lock the database for writing. Saves changes atomically on drop.
    pub fn write(&self) -> EncryptedAtomicDatabaseWrite<'_, T> {
        let key = *self.key.read();
        let salt = *self.salt.read();
        EncryptedAtomicDatabaseWrite {
            path: self.path.as_ref(),
            tmp: self.tmp.as_ref(),
            data: self.data.write(),
            key,
            salt,
        }
    }

    /// Change the password (re-encrypt with a new salt+key).
    pub fn change_password(&self, new_password: &str) -> io::Result<()> {
        let data_guard = self.data.read();

        let mut new_salt = [0u8; SALT_LEN];
        OsRng.fill_bytes(&mut new_salt);
        let new_key = derive_key(new_password, &new_salt)?;

        atomic_write_encrypted(&self.tmp, &self.path, &*data_guard, &new_key, new_salt)?;

        {
            let mut key_lock = self.key.write();
            *key_lock = new_key;
        }
        {
            let mut salt_lock = self.salt.write();
            *salt_lock = new_salt;
        }

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
    key: &Key<Aes256Gcm>,
    salt: [u8; SALT_LEN],
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
        let key = self.key.read();
        let salt = self.salt.read();
        if let Err(e) = atomic_write_encrypted(&self.tmp, &self.path, &*data_guard, &key, *salt) {
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
    key: Key<Aes256Gcm>,
    salt: [u8; SALT_LEN],
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
            atomic_write_encrypted(self.tmp, self.path, &*self.data, &self.key, self.salt)
        {
            error!("Failed to save database: {}", e);
        }
    }
}
