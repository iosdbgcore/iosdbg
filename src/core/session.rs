use std::path::{Path, PathBuf};
use std::process::Command;

use crate::core::types::{describe_attach_target, SessionLifecycle, SessionStateMachine};
use crate::types::{
    CoreError, CoreResult, RemoteConfig, RemoteErrorKind, RemoteSessionState, RemoteSessionStatus,
};

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

#[derive(Debug, Clone)]
pub struct RemoteSessionManager {
    status: RemoteSessionStatus,
    retry_remaining: u8,
    retry_delay_ms: u64,
}

impl RemoteSessionManager {
    pub fn new() -> Self {
        Self {
            status: RemoteSessionStatus::disconnected(),
            retry_remaining: 0,
            retry_delay_ms: 200,
        }
    }

    pub fn status(&self) -> RemoteSessionStatus {
        self.status.clone()
    }

    pub fn begin_connect(&mut self, config: &RemoteConfig) {
        self.retry_remaining = config.retry_count;
        self.status = RemoteSessionStatus {
            state: RemoteSessionState::Connecting,
            endpoint: Some(config.endpoint.clone()),
            session_id: None,
            error: None,
            message: "Connecting to x64dbg remote endpoint".to_string(),
        };
    }

    pub fn mark_connected(&mut self, config: &RemoteConfig, session_id: String) {
        self.retry_remaining = config.retry_count;
        self.status = RemoteSessionStatus {
            state: RemoteSessionState::Connected,
            endpoint: Some(config.endpoint.clone()),
            session_id: Some(session_id),
            error: None,
            message: "Remote session connected".to_string(),
        };
    }

    pub fn mark_failed(
        &mut self,
        config: &RemoteConfig,
        error: RemoteErrorKind,
        message: impl Into<String>,
    ) {
        self.status = RemoteSessionStatus {
            state: RemoteSessionState::Degraded,
            endpoint: Some(config.endpoint.clone()),
            session_id: None,
            error: Some(error),
            message: message.into(),
        };
    }

    pub fn can_retry(&mut self) -> bool {
        if self.retry_remaining == 0 {
            return false;
        }
        self.retry_remaining -= 1;
        true
    }

    pub fn retry_delay_ms(&self) -> u64 {
        self.retry_delay_ms
    }

    pub fn disconnect(&mut self) {
        self.retry_remaining = 0;
        self.status = RemoteSessionStatus::disconnected();
    }
}

#[cfg(test)]
mod tests {
    use super::RemoteSessionManager;
    use crate::types::{RemoteConfig, RemoteErrorKind, RemoteSessionState};

    #[test]
    fn remote_session_transitions_from_connecting_to_connected() {
        let mut manager = RemoteSessionManager::new();
        let config = RemoteConfig {
            endpoint: "mock://xdbg".to_string(),
            retry_count: 2,
            ..RemoteConfig::default()
        };

        manager.begin_connect(&config);
        assert_eq!(manager.status().state, RemoteSessionState::Connecting);
        manager.mark_connected(&config, "session-1".to_string());
        assert_eq!(manager.status().state, RemoteSessionState::Connected);
    }

    #[test]
    fn remote_session_retry_budget_is_consumed() {
        let mut manager = RemoteSessionManager::new();
        let config = RemoteConfig {
            retry_count: 1,
            ..RemoteConfig::default()
        };

        manager.begin_connect(&config);
        assert!(manager.can_retry());
        assert!(!manager.can_retry());
    }

    #[test]
    fn remote_session_failure_marks_degraded() {
        let mut manager = RemoteSessionManager::new();
        let config = RemoteConfig::default();
        manager.begin_connect(&config);
        manager.mark_failed(&config, RemoteErrorKind::Timeout, "timeout");

        let status = manager.status();
        assert_eq!(status.state, RemoteSessionState::Degraded);
        assert_eq!(status.error, Some(RemoteErrorKind::Timeout));
    }
}
