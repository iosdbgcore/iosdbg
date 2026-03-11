use rust_lldb_visual_debugger::core::engine::{DebugEngine, MockLldbEngine};
use rust_lldb_visual_debugger::types::{RemoteConfig, RemoteErrorKind, RemoteSessionState};

static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn remote_connect_success_path_reaches_connected_state() {
    let mut engine = MockLldbEngine::new().expect("engine should initialize");
    let status = engine
        .connect_remote(RemoteConfig {
            endpoint: "mock://xdbg".to_string(),
            ..RemoteConfig::default()
        })
        .expect("remote connect should return status");

    assert_eq!(status.state, RemoteSessionState::Connected);
    let pc = engine.step_in().expect("step should work in remote mode");
    assert!(pc.is_some());
}

#[test]
fn remote_connect_timeout_is_classified() {
    let _guard = ENV_LOCK.lock().expect("env lock should be available");
    std::env::set_var("IOSDBG_REMOTE_FORCE_TIMEOUT", "1");

    let mut engine = MockLldbEngine::new().expect("engine should initialize");
    let status = engine
        .connect_remote(RemoteConfig {
            endpoint: "127.0.0.1:27400".to_string(),
            retry_count: 0,
            ..RemoteConfig::default()
        })
        .expect("remote connect should return degraded status");

    std::env::remove_var("IOSDBG_REMOTE_FORCE_TIMEOUT");

    assert_eq!(status.state, RemoteSessionState::Degraded);
    assert_eq!(status.error, Some(RemoteErrorKind::Timeout));
}

#[test]
fn remote_connect_auth_failure_is_classified() {
    let mut engine = MockLldbEngine::new().expect("engine should initialize");
    let status = engine
        .connect_remote(RemoteConfig {
            endpoint: "mock://xdbg".to_string(),
            token: Some("bad-token".to_string()),
            retry_count: 0,
            ..RemoteConfig::default()
        })
        .expect("remote connect should return degraded status");

    assert_eq!(status.state, RemoteSessionState::Degraded);
    assert_eq!(status.error, Some(RemoteErrorKind::AuthFailed));
}

#[test]
fn remote_disconnect_then_reconnect_restores_session() {
    let mut engine = MockLldbEngine::new().expect("engine should initialize");

    let _ = engine
        .connect_remote(RemoteConfig {
            endpoint: "mock://xdbg".to_string(),
            ..RemoteConfig::default()
        })
        .expect("first connect should work");
    engine.disconnect_remote();
    assert_eq!(
        engine.remote_session_status().state,
        RemoteSessionState::Disconnected
    );

    let status = engine
        .connect_remote(RemoteConfig {
            endpoint: "mock://xdbg".to_string(),
            ..RemoteConfig::default()
        })
        .expect("reconnect should work");
    assert_eq!(status.state, RemoteSessionState::Connected);
}

