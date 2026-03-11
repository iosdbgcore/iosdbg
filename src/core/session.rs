use std::path::{Path, PathBuf};
use std::process::Command;

use crate::core::types::{describe_attach_target, SessionLifecycle, SessionStateMachine};
use crate::types::{CoreError, CoreResult};

#[derive(Debug, Clone)]
pub struct LldbSession {
    initialized: bool,
    target_path: Option<PathBuf>,
    attached_target: Option<String>,
    lifecycle: SessionStateMachine,
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
            attached_target: None,
            lifecycle: SessionStateMachine::new(),
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

    pub fn lifecycle(&self) -> SessionLifecycle {
        self.lifecycle.lifecycle()
    }

    pub fn detach(&mut self) {
        self.attached_target = None;
        let _ = self.lifecycle.mark_detached();
    }

    pub fn attach_to_process(
        &mut self,
        request: &crate::types::AttachRequest,
    ) -> CoreResult<String> {
        if !self.initialized {
            return Err(CoreError::new("LLDB session is not initialized"));
        }

        self.lifecycle
            .begin_attach()
            .map_err(|error| CoreError::new(format!("Attach state transition failed: {error}")))?;
        let target_label = describe_attach_target(request);
        let has_permission = has_attach_permission();
        if !has_permission {
            let _ = self.lifecycle.mark_failed();
            return Err(CoreError::new(
                "Permission denied while trying to attach (task_for_pid)",
            ));
        }

        self.attached_target = Some(target_label.clone());
        self.lifecycle
            .mark_attached()
            .map_err(|error| CoreError::new(format!("Attach state transition failed: {error}")))?;
        Ok(target_label)
    }
}

fn has_attach_permission() -> bool {
    // Test hook to validate permission-denied flows in CI without platform-specific setup.
    if std::env::var("IOSDBG_ATTACH_PERMISSION")
        .map(|value| value.eq_ignore_ascii_case("deny"))
        .unwrap_or(false)
    {
        return false;
    }

    if cfg!(target_os = "macos") {
        // On macOS, treat LLDB being available as a minimal proxy for attach capability.
        return Command::new("xcrun")
            .args(["--find", "lldb"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
    }

    true
}
