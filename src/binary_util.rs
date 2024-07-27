use anyhow::Context;
use base64::{engine::general_purpose as base64_engine, Engine as _};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Binary {
    data: Vec<u8>,
}

impl Binary {
    /// Creates a new `BinaryUtil` instance from a `Vec<u8>`
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Reads a binary file and returns a `BinaryUtil` instance.
    pub fn from_file(path: PathBuf) -> anyhow::Result<Self> {
        let data = fs::read(path)?;

        return Ok(Self { data });
    }

    /// Decode a base64-encoded string and return a `BinaryUtil` instance.
    pub fn from_b64(value: String) -> anyhow::Result<Self> {
        let data = base64_engine::STANDARD
            .decode(&value)
            .context("Failed to decode base64 string")?;
        return Ok(Self { data });
    }

    /// Gets the size of the binary data in bytes.
    pub fn size(&self) -> usize {
        return self.data.len();
    }

    /// Returns a clone of the binary data.
    pub fn read(&self) -> Vec<u8> {
        return self.data.clone();
    }
}

impl ToString for Binary {
    fn to_string(&self) -> String {
        format!(
            "binary util!({})",
            base64_engine::STANDARD.encode(self.data.clone())
        )
    }
}
