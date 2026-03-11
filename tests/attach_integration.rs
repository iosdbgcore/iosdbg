use rust_lldb_visual_debugger::core::engine::{DebugEngine, MockLldbEngine};
use rust_lldb_visual_debugger::types::{AttachErrorKind, AttachRequest};

static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn attach_fails_with_permission_denied_when_permission_probe_is_blocked() {
    let _guard = ENV_LOCK.lock().expect("env lock should be available");
    std::env::set_var("IOSDBG_ATTACH_PERMISSION", "deny");

    let mut engine = MockLldbEngine::new().expect("engine should initialize");
    let result = engine
        .attach_process(AttachRequest::by_pid(std::process::id()))
        .expect("attach should return structured result");

    std::env::remove_var("IOSDBG_ATTACH_PERMISSION");

    assert!(!result.attached);
    assert_eq!(result.error, Some(AttachErrorKind::PermissionDenied));
}

#[test]
fn attach_fails_with_target_not_found_for_unknown_process_name() {
    let mut engine = MockLldbEngine::new().expect("engine should initialize");
    let result = engine
        .attach_process(AttachRequest::by_process_name("__iosdbg_missing_process__"))
        .expect("attach should return structured result");

    assert!(!result.attached);
    assert_eq!(result.error, Some(AttachErrorKind::TargetNotFound));
}
