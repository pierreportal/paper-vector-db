use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

use serde::{Serialize, de::DeserializeOwned};

#[derive(Debug)]
pub struct Storage {
    path: PathBuf,
}

impl Storage {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn save<T>(&self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        let config = bincode::config::standard();

        let encoded = bincode::serde::encode_to_vec(value, config)?;

        fs::write(&self.path, encoded)?;

        Ok(())
    }

    pub fn load<T>(&self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let bytes = fs::read(&self.path)?;

        let config = bincode::config::standard();

        let (value, _) = bincode::serde::decode_from_slice(&bytes, config)?;

        Ok(value)
    }

    pub fn exists(&self) -> bool {
        Path::new(&self.path).exists()
    }
}
