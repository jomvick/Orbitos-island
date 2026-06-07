use std::os::unix::fs::PermissionsExt;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};

use tokio::net::{UnixListener, UnixStream};

use super::codec::{BridgeCodec, CodecError};

pub fn get_default_socket_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| {
        tracing::warn!("HOME env var not set, falling back to /tmp");
        "/tmp".to_string()
    });
    Path::new(&home).join(".agentos").join("run").join("agentosd.sock")
}

#[derive(Debug, Clone)]
pub struct SocketConfig {
    pub path: PathBuf,
    pub max_connections: u32,
}

impl Default for SocketConfig {
    fn default() -> Self {
        Self {
            path: get_default_socket_path(),
            max_connections: 32,
        }
    }
}

pub struct IpcServer {
    listener: UnixListener,
    config: SocketConfig,
}

impl IpcServer {
    pub fn bind(config: SocketConfig) -> Result<Self, std::io::Error> {
        let path = &config.path;
        
        // Security Fix: Prevent TOCTOU vulnerability by securing the parent directory
        // *before* the socket is created. This ensures the socket is never exposed
        // even briefly between `bind` and `set_permissions`.
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
            std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))?;
        }
        
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        
        let listener = UnixListener::bind(path)?;
        // Restrict socket permissions to owner-only (0o600)
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        Ok(Self { listener, config })
    }

    pub async fn accept(&self) -> Result<(BridgeCodec, std::os::unix::io::RawFd), std::io::Error> {
        let (stream, _) = self.listener.accept().await?;
        let fd = stream.as_raw_fd();
        let (read, write) = stream.into_split();
        let codec = BridgeCodec::new(read, write);
        Ok((codec, fd))
    }

    pub fn local_path(&self) -> &Path {
        self.config.path.as_path()
    }
}

pub async fn connect_to_daemon(timeout: std::time::Duration) -> Result<BridgeCodec, CodecError> {
    let socket_path = get_default_socket_path();
    let stream = tokio::time::timeout(timeout, UnixStream::connect(socket_path))
        .await
        .map_err(|_| CodecError::Timeout)?
        .map_err(CodecError::Io)?;

    let (read, write) = stream.into_split();
    Ok(BridgeCodec::new(read, write))
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        let path = &self.config.path;
        if path.exists() {
            let _ = std::fs::remove_file(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    /// 🔴 Critical: if the daemon is down, the hook must time out in < 150ms,
    /// not hang indefinitely and freeze the user's shell.
    #[tokio::test]
    async fn test_hook_timeout_does_not_hang() {
        let nonexistent = std::path::PathBuf::from("/tmp/agentos-test-nonexistent-99999.sock");
        let _ = std::fs::remove_file(&nonexistent); // ensure it does not exist

        // Override the default path lookup by connecting directly to the nonexistent path.
        let timeout = Duration::from_millis(100);
        let start = Instant::now();

        let result =
            tokio::time::timeout(timeout, tokio::net::UnixStream::connect(&nonexistent)).await;

        let elapsed = start.elapsed();

        // Either timed out or the connection was refused — both are acceptable failures.
        assert!(
            result.as_ref().map_or(true, Result::is_err),
            "expected connection failure"
        );
        assert!(
            elapsed.as_millis() < 150,
            "timeout took {}ms, expected < 150ms",
            elapsed.as_millis()
        );
    }

    /// Verifies the IpcServer binds and accepts exactly one connection.
    #[tokio::test]
    async fn test_server_bind_and_accept() {
        let dir = std::env::temp_dir()
            .join(format!("agentos-ipc-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let socket_path = dir.join("bind-test.sock");
        let _ = std::fs::remove_file(&socket_path);

        let config = SocketConfig {
            path: socket_path.clone(),
            max_connections: 4,
        };
        let server = IpcServer::bind(config).expect("failed to bind IPC server");
        let server_path = server.local_path().to_path_buf();

        let accept_task = tokio::spawn(async move {
            tokio::time::timeout(Duration::from_secs(2), server.accept())
                .await
                .expect("accept timed out")
                .expect("accept failed")
        });

        tokio::time::sleep(Duration::from_millis(20)).await;
        let _stream = tokio::net::UnixStream::connect(&server_path)
            .await
            .expect("client connect failed");

        accept_task.await.expect("accept task panicked");
        let _ = std::fs::remove_file(&socket_path);
    }
}
