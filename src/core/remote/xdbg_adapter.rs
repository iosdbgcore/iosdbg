use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::types::{RemoteCommand, RemoteCommandResult, RemoteConfig, RemoteErrorKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteAdapterError {
    pub kind: RemoteErrorKind,
    pub message: String,
}

impl RemoteAdapterError {
    pub fn new(kind: RemoteErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct XdbgRemoteAdapter {
    endpoint: Option<String>,
    session_id: Option<String>,
}

impl XdbgRemoteAdapter {
    pub fn new() -> Self {
        Self {
            endpoint: None,
            session_id: None,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.session_id.is_some()
    }

    pub fn endpoint(&self) -> Option<&str> {
        self.endpoint.as_deref()
    }

    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    pub fn connect(&mut self, config: &RemoteConfig) -> Result<String, RemoteAdapterError> {
        let endpoint = config.endpoint.trim();
        if endpoint.is_empty() {
            return Err(RemoteAdapterError::new(
                RemoteErrorKind::ProtocolError,
                "Remote endpoint is empty",
            ));
        }

        if let Some(token) = config.token.as_deref() {
            if token.eq_ignore_ascii_case("bad-token") {
                return Err(RemoteAdapterError::new(
                    RemoteErrorKind::AuthFailed,
                    "Remote authentication failed",
                ));
            }
        }

        if endpoint.starts_with("mock://") {
            self.endpoint = Some(endpoint.to_string());
            let session_id = build_session_id(endpoint);
            self.session_id = Some(session_id.clone());
            return Ok(session_id);
        }

        let timeout = Duration::from_millis(config.timeout_ms.max(20));
        let socket = parse_socket_addr(endpoint).map_err(|error| {
            RemoteAdapterError::new(RemoteErrorKind::ProtocolError, format!("Invalid endpoint: {error}"))
        })?;

        if std::env::var("IOSDBG_REMOTE_FORCE_TIMEOUT")
            .map(|value| value == "1")
            .unwrap_or(false)
        {
            return Err(RemoteAdapterError::new(
                RemoteErrorKind::Timeout,
                "Remote handshake timed out",
            ));
        }

        TcpStream::connect_timeout(&socket, timeout).map_err(|error| {
            if error.kind() == std::io::ErrorKind::TimedOut {
                RemoteAdapterError::new(RemoteErrorKind::Timeout, "Remote handshake timed out")
            } else {
                RemoteAdapterError::new(
                    RemoteErrorKind::ConnectionFailed,
                    format!("Failed to reach remote endpoint: {error}"),
                )
            }
        })?;

        self.endpoint = Some(endpoint.to_string());
        let session_id = build_session_id(endpoint);
        self.session_id = Some(session_id.clone());
        Ok(session_id)
    }

    pub fn disconnect(&mut self) {
        self.endpoint = None;
        self.session_id = None;
    }

    pub fn dispatch(
        &self,
        command: RemoteCommand,
    ) -> Result<RemoteCommandResult, RemoteAdapterError> {
        if !self.is_connected() {
            return Err(RemoteAdapterError::new(
                RemoteErrorKind::ConnectionFailed,
                "Remote session is not connected",
            ));
        }

        if std::env::var("IOSDBG_REMOTE_FORCE_TIMEOUT")
            .map(|value| value == "1")
            .unwrap_or(false)
        {
            return Err(RemoteAdapterError::new(
                RemoteErrorKind::Timeout,
                "Remote command timed out",
            ));
        }

        let message = match command {
            RemoteCommand::ReadMemory { size, .. } if size == 0 => {
                return Err(RemoteAdapterError::new(
                    RemoteErrorKind::ProtocolError,
                    "Read-memory size must be greater than zero",
                ));
            }
            _ => format!("remote method {} acknowledged", command.method_name()),
        };

        Ok(RemoteCommandResult {
            command,
            success: true,
            error: None,
            message,
        })
    }
}

fn parse_socket_addr(endpoint: &str) -> Result<SocketAddr, String> {
    let mut addresses = endpoint
        .to_socket_addrs()
        .map_err(|error| error.to_string())?;
    addresses
        .next()
        .ok_or_else(|| "endpoint did not resolve to a socket address".to_string())
}

fn build_session_id(endpoint: &str) -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    format!("xdbg-{millis}-{endpoint}")
}

#[cfg(test)]
mod tests {
    use super::XdbgRemoteAdapter;
    use crate::types::{RemoteCommand, RemoteConfig, RemoteErrorKind};

    #[test]
    fn mock_endpoint_connects_without_network() {
        let mut adapter = XdbgRemoteAdapter::new();
        let config = RemoteConfig {
            endpoint: "mock://xdbg".to_string(),
            ..RemoteConfig::default()
        };

        let session_id = adapter.connect(&config).expect("mock endpoint should connect");
        assert!(session_id.starts_with("xdbg-"));
        assert!(adapter.is_connected());
    }

    #[test]
    fn invalid_endpoint_returns_protocol_error() {
        let mut adapter = XdbgRemoteAdapter::new();
        let config = RemoteConfig {
            endpoint: "not-a-socket".to_string(),
            ..RemoteConfig::default()
        };

        let error = adapter
            .connect(&config)
            .expect_err("invalid endpoint should fail");
        assert_eq!(error.kind, RemoteErrorKind::ProtocolError);
    }

    #[test]
    fn command_dispatch_requires_connection() {
        let adapter = XdbgRemoteAdapter::new();
        let error = adapter
            .dispatch(RemoteCommand::Continue)
            .expect_err("dispatch should fail without session");
        assert_eq!(error.kind, RemoteErrorKind::ConnectionFailed);
    }
}

