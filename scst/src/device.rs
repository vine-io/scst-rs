use std::ffi::OsStr;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{Layer, read_fl, read_link};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Device {
    #[serde(skip)]
    root: String,
    name: String,
    handler: String,
    filename: String,
    active: i8,
    read_only: i8,
    size: usize,
    blocksize: u32,
}

impl Device {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn handler(&self) -> &str {
        &self.handler
    }

    pub fn filename(&self) -> &Path {
        Path::new(&self.filename)
    }

    pub fn is_active(&self) -> bool {
        self.active == 1
    }

    pub fn read_only(&self) -> bool {
        self.read_only == 1
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn blocksize(&self) -> u32 {
        self.blocksize
    }
}

impl Layer for Device {
    fn root(&self) -> &Path {
        Path::new(&self.root)
    }

    fn load<P>(&mut self, root: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let root_ref = root.as_ref();
        self.root = root_ref.to_string_lossy().to_string();
        self.name = root_ref
            .file_name()
            .unwrap_or(OsStr::new(""))
            .to_string_lossy()
            .to_string();
        self.handler = read_link(root_ref.join("handler"))?
            .file_name()
            .unwrap_or(OsStr::new(""))
            .to_string_lossy()
            .to_string();
        self.filename = read_fl(root_ref.join("filename"))?;
        self.active = read_fl(root_ref.join("active"))?.parse::<i8>()?;
        self.read_only = read_fl(root_ref.join("read_only"))?.parse::<i8>()?;
        self.size = read_fl(root_ref.join("size"))?.parse::<usize>()?;
        self.blocksize = read_fl(root_ref.join("blocksize"))?.parse::<u32>()?;

        Ok(())
    }
}

#[cfg(test)]
mod test {}
