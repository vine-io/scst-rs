use std::ffi::OsStr;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{Layer, Target};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CopyManager {
    #[serde(skip)]
    root: String,
    name: String,

    tgt: Target,
}

impl CopyManager {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tgt(&self) -> &Target {
        &self.tgt
    }
}

impl Layer for CopyManager {
    fn root(&self) -> &std::path::Path {
        Path::new(&self.root)
    }

    fn load<P: AsRef<std::path::Path>>(&mut self, root: P) -> Result<()> {
        let root_ref = root.as_ref();
        self.root = root_ref.to_string_lossy().to_string();
        self.name = root_ref
            .file_name()
            .unwrap_or(OsStr::new(""))
            .to_string_lossy()
            .to_string();

        let mut target = Target::default();
        target.set_name("copy_manager_tgt");
        target.load(root_ref.join("copy_manager_tgt"))?;
        self.tgt = target;

        Ok(())
    }
}
