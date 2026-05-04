//! Persistent storage of the 32-byte ed25519 seed.
//!
//! Two backends:
//!
//! - [`KeyringStore`] uses the OS keyring (libsecret on Linux, Keychain on
//!   macOS, Credential Manager on Windows).
//! - [`FileStore`] writes the hex-encoded seed to a flat file in
//!   `$XDG_CONFIG_HOME/torrents-trackers/identity.key` with mode 0600.
//!
//! [`DefaultStore`] tries keyring first and falls back to file storage if
//! the keyring backend isn't available (e.g. headless servers, CI).

use std::path::PathBuf;

use zeroize::Zeroize;

use crate::error::{IdentityError, Result};

const KEYRING_SERVICE: &str = "torrents-trackers";
const KEYRING_USER: &str = "identity";
const FILE_NAME: &str = "identity.key";

pub trait IdentityStore: Send + Sync {
    fn store(&self, seed: &[u8; 32]) -> Result<()>;
    fn load(&self) -> Result<Option<[u8; 32]>>;
    fn delete(&self) -> Result<()>;
}

// --------------------------------------------------------------------------
// Keyring backend
// --------------------------------------------------------------------------

pub struct KeyringStore;

impl KeyringStore {
    fn entry() -> Result<keyring::Entry> {
        keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
            .map_err(|e| IdentityError::Keyring(e.to_string()))
    }
}

impl IdentityStore for KeyringStore {
    fn store(&self, seed: &[u8; 32]) -> Result<()> {
        let entry = Self::entry()?;
        let mut hex = hex::encode(seed);
        let res = entry
            .set_password(&hex)
            .map_err(|e| IdentityError::Keyring(e.to_string()));
        hex.zeroize();
        res
    }

    fn load(&self) -> Result<Option<[u8; 32]>> {
        let entry = Self::entry()?;
        match entry.get_password() {
            Ok(mut hex) => {
                let bytes = hex::decode(&hex).map_err(IdentityError::Hex)?;
                hex.zeroize();
                let arr: [u8; 32] = bytes
                    .try_into()
                    .map_err(|v: Vec<u8>| IdentityError::InvalidSeedLength(v.len()))?;
                Ok(Some(arr))
            }
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(IdentityError::Keyring(e.to_string())),
        }
    }

    fn delete(&self) -> Result<()> {
        let entry = Self::entry()?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(IdentityError::Keyring(e.to_string())),
        }
    }
}

// --------------------------------------------------------------------------
// File backend
// --------------------------------------------------------------------------

pub struct FileStore {
    path: PathBuf,
}

impl FileStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn default_path() -> Result<PathBuf> {
        let base = dirs::config_dir().ok_or(IdentityError::NoDataDir)?;
        Ok(base.join("torrents-trackers").join(FILE_NAME))
    }

    pub fn at_default() -> Result<Self> {
        Ok(Self::new(Self::default_path()?))
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl IdentityStore for FileStore {
    fn store(&self, seed: &[u8; 32]) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut hex = hex::encode(seed);
        std::fs::write(&self.path, &hex)?;
        hex.zeroize();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&self.path, perms)?;
        }
        Ok(())
    }

    fn load(&self) -> Result<Option<[u8; 32]>> {
        if !self.path.exists() {
            return Ok(None);
        }
        let mut hex = std::fs::read_to_string(&self.path)?;
        let trimmed = hex.trim();
        let bytes = hex::decode(trimmed).map_err(IdentityError::Hex)?;
        hex.zeroize();
        let arr: [u8; 32] = bytes
            .try_into()
            .map_err(|v: Vec<u8>| IdentityError::InvalidSeedLength(v.len()))?;
        Ok(Some(arr))
    }

    fn delete(&self) -> Result<()> {
        match std::fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

// --------------------------------------------------------------------------
// DefaultStore — file-backed
// --------------------------------------------------------------------------

/// Default identity storage. Uses [`FileStore`] under
/// `$XDG_CONFIG_HOME/torrents-trackers/identity.key` (mode 0600 on Unix).
///
/// In Phase 2 we deliberately do not use [`KeyringStore`] by default: the
/// `keyring` crate v3 ships without a built-in backend, and pulling in a
/// platform-specific feature requires runtime services (D-Bus / secret-service
/// on Linux, Keychain access on macOS) that complicate headless installs and
/// CI. The file backend is good enough for the MVP — Phase 3+ will wire up
/// a real keyring under a feature flag.
pub struct DefaultStore {
    file: FileStore,
}

impl DefaultStore {
    pub fn new() -> Result<Self> {
        Ok(Self {
            file: FileStore::at_default()?,
        })
    }
}

impl IdentityStore for DefaultStore {
    fn store(&self, seed: &[u8; 32]) -> Result<()> {
        self.file.store(seed)
    }

    fn load(&self) -> Result<Option<[u8; 32]>> {
        self.file.load()
    }

    fn delete(&self) -> Result<()> {
        self.file.delete()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_store_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("id.key");
        let store = FileStore::new(path.clone());
        assert!(store.load().unwrap().is_none());
        let seed = [11u8; 32];
        store.store(&seed).unwrap();
        assert_eq!(store.load().unwrap(), Some(seed));
        store.delete().unwrap();
        assert!(store.load().unwrap().is_none());
    }

    #[cfg(unix)]
    #[test]
    fn file_store_uses_0600_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("id.key");
        let store = FileStore::new(path.clone());
        store.store(&[1u8; 32]).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }
}
