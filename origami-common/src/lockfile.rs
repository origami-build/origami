use std::io::{Read, Write};

use binserde::{BinDeserialize, BinSerialize};

#[derive(Debug, Default, BinSerialize, BinDeserialize)]
pub struct WorkspaceLock {
    pub packages: Vec<Package>,
}

impl WorkspaceLock {
    pub fn write<W: Write>(&self, pipe: W) -> binserde::Result<()> {
        binserde::serialize_into(pipe, self)
    }

    pub fn read<R: Read>(pipe: R) -> binserde::Result<Self> {
        binserde::deserialize_from(pipe)
    }
}

#[derive(Debug, BinSerialize, BinDeserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub sources: Vec<RemoteFile>,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone, BinSerialize, BinDeserialize)]
pub enum RemoteFile {
    Http {
        source: String,
        checksum: [u8; 32],
        size: usize,
    },
}

#[derive(Debug, BinSerialize, BinDeserialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
}
