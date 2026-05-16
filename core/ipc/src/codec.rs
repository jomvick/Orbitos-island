use std::io;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};

use super::messages::IpcMessage;

pub const MAX_MESSAGE_SIZE: usize = 1_048_576; // 1 MiB

pub struct BridgeCodec {
    reader: BufReader<OwnedReadHalf>,
    writer: OwnedWriteHalf,
}

#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("message too large: {0} bytes")]
    MessageTooLarge(usize),
    #[error("connection closed")]
    ConnectionClosed,
    #[error("timeout")]
    Timeout,
}

impl BridgeCodec {
    pub fn new(read: OwnedReadHalf, write: OwnedWriteHalf) -> Self {
        Self {
            reader: BufReader::new(read),
            writer: write,
        }
    }

    pub async fn send(&mut self, msg: &IpcMessage) -> Result<(), CodecError> {
        let mut buf = serde_json::to_vec(msg)?;
        buf.push(b'\n');
        self.writer.write_all(&buf).await?;
        self.writer.flush().await?;
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<IpcMessage, CodecError> {
        let mut line = String::new();
        let n = self.reader.read_line(&mut line).await?;
        if n == 0 {
            return Err(CodecError::ConnectionClosed);
        }
        if n > MAX_MESSAGE_SIZE {
            return Err(CodecError::MessageTooLarge(n));
        }
        let msg: IpcMessage = serde_json::from_str(line.trim())?;
        Ok(msg)
    }

    pub async fn recv_timeout(
        &mut self,
        timeout: std::time::Duration,
    ) -> Result<IpcMessage, CodecError> {
        tokio::time::timeout(timeout, self.recv())
            .await
            .map_err(|_| CodecError::Timeout)?
    }

    pub fn split(self) -> (BufReader<OwnedReadHalf>, OwnedWriteHalf) {
        (self.reader, self.writer)
    }
}
