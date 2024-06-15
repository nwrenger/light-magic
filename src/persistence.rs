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

/// Trait `DB` that requires a few implementations and a `save` and `load` method
pub trait DB<'de>: Default + Serialize {
    fn load(file: impl io::Read) -> std::io::Result<Self>
    where
        Self: Sized,
        Self: DeserializeOwned,
    {
        Ok(serde_json::from_reader(file)?)
    }
    fn save(&self, file: impl io::Write) -> std::io::Result<()> {
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }
}

/// Synchronized Wrapper, that automatically saves changes
pub struct AtomicDatabase<T: for<'a> DB<'a>> {
    path: PathBuf,
    /// Name of the DB temporary file
    tmp: PathBuf,
    data: RwLock<T>,
}

impl<T: for<'a> DB<'a> + DeserializeOwned> AtomicDatabase<T> {
    /// Load the database from the file system.
    ///
    /// This also migrates it if necessary.
    pub fn load(path: &Path) -> Result<Self, std::io::Error> {
        let new_path = path.with_extension("json");
        let tmp = Self::tmp_path(&new_path)?;

        let file = File::open(path)?;
        // for the future: make here version checks
        let data = T::load(file)?;
        atomic_write(&tmp, &new_path, &data)?;

        Ok(Self {
            path: new_path,
            tmp,
            data: RwLock::new(data),
        })
    }

    /// Create a new database and save it.
    pub fn create(path: &Path) -> Result<Self, std::io::Error> {
        let tmp = Self::tmp_path(path)?;

        let data = Default::default();
        atomic_write(&tmp, path, &data)?;

        Ok(Self {
            path: path.into(),
            tmp,
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
            path: &self.path,
            tmp: &self.tmp,
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
fn atomic_write<T: for<'a> DB<'a>>(
    tmp: &Path,
    path: &Path,
    data: &T,
) -> Result<(), std::io::Error> {
    {
        let mut tmpfile = File::create(tmp)?;
        data.save(&mut tmpfile)?;
        tmpfile.sync_all()?; // just to be sure!
    }
    fs::rename(tmp, path)?;
    Ok(())
}

impl<T: for<'a> DB<'a>> fmt::Debug for AtomicDatabase<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AtomicDatabase")
            .field("file", &self.path)
            .finish()
    }
}

impl<T: for<'a> DB<'a>> Drop for AtomicDatabase<T> {
    fn drop(&mut self) {
        info!("Saving database");
        let guard = self.data.read().unwrap();
        atomic_write(&self.tmp, &self.path, &*guard).unwrap();
    }
}

pub struct AtomicDatabaseRead<'a, T: for<'b> DB<'b>> {
    data: RwLockReadGuard<'a, T>,
}

impl<'a, T: for<'b> DB<'b>> Deref for AtomicDatabaseRead<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

pub struct AtomicDatabaseWrite<'a, T: for<'b> DB<'b>> {
    tmp: &'a Path,
    path: &'a Path,
    data: RwLockWriteGuard<'a, T>,
}

impl<'a, T: for<'b> DB<'b>> Deref for AtomicDatabaseWrite<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a, T: for<'b> DB<'b>> DerefMut for AtomicDatabaseWrite<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<'a, T: for<'b> DB<'b>> Drop for AtomicDatabaseWrite<'a, T> {
    fn drop(&mut self) {
        info!("Saving database");
        atomic_write(self.tmp, self.path, &*self.data).unwrap();
    }
}
