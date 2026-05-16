use std::os::unix::fs::PermissionsExt;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};

use tokio::net::{UnixListener, UnixStream};

use super::codec::{BridgeCodec, CodecError};

pub fn get_default_socket_path() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME env var not set");
    Path::new(&home).join(".agentosd.sock")
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
