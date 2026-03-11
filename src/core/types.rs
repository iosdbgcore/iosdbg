use crate::types::{AttachErrorKind, AttachRequest, AttachTarget};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionLifecycle {
    Detached,
    Attaching,
    Attached,
    Failed,
}

impl Default for SessionLifecycle {
    fn default() -> Self {
        Self::Detached
    }
}

#[derive(Debug, Clone)]
pub struct SessionStateMachine {
    lifecycle: SessionLifecycle,
}

impl SessionStateMachine {
    pub fn new() -> Self {
        Self {
            lifecycle: SessionLifecycle::Detached,
        }
    }

    pub fn lifecycle(&self) -> SessionLifecycle {
        self.lifecycle
    }

    pub fn begin_attach(&mut self) -> Result<(), String> {
        self.transition(SessionLifecycle::Attaching)
    }

    pub fn mark_attached(&mut self) -> Result<(), String> {
        self.transition(SessionLifecycle::Attached)
    }

    pub fn mark_failed(&mut self) -> Result<(), String> {
        self.transition(SessionLifecycle::Failed)
    }

    pub fn mark_detached(&mut self) -> Result<(), String> {
        self.transition(SessionLifecycle::Detached)
    }

    fn transition(&mut self, next: SessionLifecycle) -> Result<(), String> {
        let is_valid = matches!(
            (self.lifecycle, next),
            (SessionLifecycle::Detached, SessionLifecycle::Attaching)
                | (SessionLifecycle::Attaching, SessionLifecycle::Attached)
                | (SessionLifecycle::Attaching, SessionLifecycle::Failed)
                | (SessionLifecycle::Attached, SessionLifecycle::Detached)
                | (SessionLifecycle::Attached, SessionLifecycle::Attaching)
                | (SessionLifecycle::Failed, SessionLifecycle::Detached)
                | (SessionLifecycle::Failed, SessionLifecycle::Attaching)
        ) || self.lifecycle == next;

        if !is_valid {
            return Err(format!(
                "Invalid lifecycle transition: {:?} -> {:?}",
                self.lifecycle, next
            ));
        }

        self.lifecycle = next;
        Ok(())
    }
}

pub fn describe_attach_target(request: &AttachRequest) -> String {
    match &request.target {
        AttachTarget::Pid(pid) => format!("pid:{pid}"),
        AttachTarget::ProcessName(name) => format!("name:{name}"),
    }
}

pub fn validate_attach_request(request: &AttachRequest) -> Result<(), AttachErrorKind> {
    match &request.target {
        AttachTarget::Pid(pid) => {
            if *pid == 0 {
                return Err(AttachErrorKind::TargetNotFound);
            }
            Ok(())
        }
        AttachTarget::ProcessName(name) => {
            if name.trim().is_empty() {
                return Err(AttachErrorKind::TargetNotFound);
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SessionLifecycle, SessionStateMachine};

    #[test]
    fn session_state_machine_validates_transition_order() {
        let mut machine = SessionStateMachine::new();
        machine
            .begin_attach()
            .expect("detached -> attaching should be valid");
        machine
            .mark_attached()
            .expect("attaching -> attached should be valid");
        machine
            .mark_detached()
            .expect("attached -> detached should be valid");
    }

    #[test]
    fn session_state_machine_rejects_invalid_transition() {
        let mut machine = SessionStateMachine::new();
        let error = machine
            .mark_attached()
            .expect_err("detached -> attached should fail");
        assert!(error.contains("Invalid lifecycle transition"));
        assert_eq!(machine.lifecycle(), SessionLifecycle::Detached);
    }
}
