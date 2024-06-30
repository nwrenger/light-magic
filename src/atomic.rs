use serde::{de::DeserializeOwned, Serialize};
use std::{
    ffi::{OsStr, OsString},
    fmt,
    fs::{self, File},
    io::{self, BufWriter},
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};
use tracing::{error, info};

/// Trait DataStore that requires a few implementations.
/// The defined functions have default definitions.
pub trait DataStore: Default + Serialize {
    /// Opens a Database by the specified path. If the Database doesn't exist, this will create a new one! Wrap a `Arc<_>` around it to use it in parallel contexts!
    fn open<P>(db: P) -> AtomicDatabase<Self>
    where
        P: AsRef<Path>,
        Self: DeserializeOwned,
    {
        let db_path = db.as_ref();
        if db_path.exists() {
            AtomicDatabase::load(db_path).unwrap()
        } else {
            AtomicDatabase::create(db_path).unwrap()
        }
    }

    /// Creates a Database instance in memory. Wrap a `Arc<_>` around it to use it in parallel contexts!
    fn open_in_memory() -> AtomicDatabase<Self>
    where
        Self: DeserializeOwned,
    {
        AtomicDatabase::load_in_memory()
    }

    /// Loads file data into the `Database`
    fn load(file: impl io::Read) -> std::io::Result<Self>
    where
        Self: Sized,
        Self: DeserializeOwned,
    {
        Ok(serde_json::from_reader(file)?)
    }

    /// Saves data of the `Database` to a file
    fn save(&self, file: impl io::Write) -> std::io::Result<()> {
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }
}

/// Synchronized Wrapper, that automatically saves changes when path and tmp are defined
pub struct AtomicDatabase<T: DataStore> {
    path: Option<PathBuf>,
    /// Name of the DataStore temporary file
    tmp: Option<PathBuf>,
    data: RwLock<T>,
}

impl<T: DataStore + DeserializeOwned> AtomicDatabase<T> {
    /// Load the database in memory.
    pub fn load_in_memory() -> Self {
        Self {
            path: None,
            tmp: None,
            data: RwLock::new(T::default()),
        }
    }

    /// Load the database from the file system.
    pub fn load(path: &Path) -> Result<Self, std::io::Error> {
        let new_path = path.with_extension("json");
        let tmp = Self::tmp_path(&new_path)?;

        let file = File::open(path)?;
        // for the future: make here version checks
        let data = T::load(file)?;
        atomic_write(&tmp, &new_path, &data)?;

        Ok(Self {
            path: Some(new_path),
            tmp: Some(tmp),
            data: RwLock::new(data),
        })
    }

    /// Create a new database and save it.
    pub fn create(path: &Path) -> Result<Self, std::io::Error> {
        let tmp = Self::tmp_path(path)?;

        let data = Default::default();
        atomic_write(&tmp, path, &data)?;

        Ok(Self {
            path: Some(path.into()),
            tmp: Some(tmp),
            data: RwLock::new(data),
        })
    }

    /// Lock the database for reading.
    pub fn read(&self) -> AtomicDatabaseRead<'_, T> {
        AtomicDatabaseRead {
            data: self.data.read().unwrap(),
        }
    }

    /// Lock the database for writing. This will save the changes atomically on drop.
    pub fn write(&self) -> AtomicDatabaseWrite<'_, T> {
        AtomicDatabaseWrite {
            path: self.path.as_deref(),
            tmp: self.tmp.as_deref(),
            data: self.data.write().unwrap(),
        }
    }

    fn tmp_path(path: &Path) -> Result<PathBuf, std::io::Error> {
        let mut tmp_name = OsString::from(".");
        tmp_name.push(path.file_name().unwrap_or(OsStr::new("db")));
        tmp_name.push("~");
        let tmp = path.with_file_name(tmp_name);
        if tmp.exists() {
            error!(
            "Found orphaned database temporary file '{tmp:?}'. The server has recently crashed or is already running. Delete this before continuing!"
        );
            return Err(std::io::Error::last_os_error());
        }
        Ok(tmp)
    }
}

/// Atomic write routine, loosely inspired by the tempfile crate.
///
/// This assumes that the rename FS operations are atomic.
fn atomic_write<T: DataStore>(tmp: &Path, path: &Path, data: &T) -> Result<(), std::io::Error> {
    {
        let mut tmpfile = File::create(tmp)?;
        data.save(&mut tmpfile)?;
        tmpfile.sync_all()?; // just to be sure!
    }
    fs::rename(tmp, path)?;
    Ok(())
}

impl<T: DataStore> fmt::Debug for AtomicDatabase<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AtomicDatabase")
            .field("file", &self.path)
            .finish()
    }
}

impl<T: DataStore> Drop for AtomicDatabase<T> {
    fn drop(&mut self) {
        if let Some(tmp) = &self.tmp {
            if let Some(path) = &self.path {
                info!("Saving database");
                let guard = self.data.read().unwrap();
                atomic_write(tmp, path, &*guard).unwrap();
            }
        }
    }
}

pub struct AtomicDatabaseRead<'a, T: DataStore> {
    data: RwLockReadGuard<'a, T>,
}

impl<'a, T: DataStore> Deref for AtomicDatabaseRead<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

pub struct AtomicDatabaseWrite<'a, T: DataStore> {
    tmp: Option<&'a Path>,
    path: Option<&'a Path>,
    data: RwLockWriteGuard<'a, T>,
}

impl<'a, T: DataStore> Deref for AtomicDatabaseWrite<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a, T: DataStore> DerefMut for AtomicDatabaseWrite<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<'a, T: DataStore> Drop for AtomicDatabaseWrite<'a, T> {
    fn drop(&mut self) {
        if let Some(tmp) = self.tmp {
            if let Some(path) = self.path {
                info!("Saving database");
                atomic_write(tmp, path, &*self.data).unwrap();
            }
        }
    }
}
