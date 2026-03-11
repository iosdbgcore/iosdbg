use std::path::{Path, PathBuf};

use crate::types::{CoreError, CoreResult};

#[derive(Debug, Clone)]
pub struct LldbSession {
    initialized: bool,
    target_path: Option<PathBuf>,
}

impl LldbSession {
    pub fn initialize() -> CoreResult<Self> {
        #[cfg(feature = "real-lldb")]
        {
            let _ = lldb_sys::LLDB_INVALID_ADDRESS;
        }

        Ok(Self {
            initialized: true,
            target_path: None,
        })
    }

    pub fn load_target(&mut self, binary_path: impl AsRef<Path>) -> CoreResult<()> {
        if !self.initialized {
            return Err(CoreError::new("LLDB session is not initialized"));
        }

        self.target_path = Some(binary_path.as_ref().to_path_buf());
        Ok(())
    }

    pub fn launch_process(&self) -> CoreResult<()> {
        if self.target_path.is_none() {
            return Err(CoreError::new("No target binary loaded"));
        }

        Ok(())
    }
}
