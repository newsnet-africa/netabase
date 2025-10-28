//! Codec implementation for sync protocol messages
//!
//! Implements libp2p's request_response codec for serializing/deserializing
//! sync messages over the wire.

use crate::sync::protocol::{SyncRequest, SyncResponse};
use async_trait::async_trait;
use futures::{prelude::*, io::{AsyncRead, AsyncWrite}};
use libp2p::request_response::Codec;
use std::io;

/// Protocol name for sync
pub const SYNC_PROTOCOL: &str = "/netabase/sync/1.0.0";

/// Sync protocol codec for request_response
#[derive(Debug, Clone, Default)]
pub struct SyncCodec;

impl SyncCodec {
    /// Maximum message size (10 MB)
    pub const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024;
}

#[async_trait]
impl Codec for SyncCodec {
    type Protocol = String;
    type Request = SyncRequest;
    type Response = SyncResponse;

    async fn read_request<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        // Read length prefix (4 bytes)
        let mut len_buf = [0u8; 4];
        io.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        // Check max size
        if len > Self::MAX_MESSAGE_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Message too large: {} bytes", len),
            ));
        }

        // Read message
        let mut buf = vec![0u8; len];
        io.read_exact(&mut buf).await?;

        // Deserialize
        serde_json::from_slice(&buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn read_response<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        // Read length prefix (4 bytes)
        let mut len_buf = [0u8; 4];
        io.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        // Check max size
        if len > Self::MAX_MESSAGE_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Message too large: {} bytes", len),
            ));
        }

        // Read message
        let mut buf = vec![0u8; len];
        io.read_exact(&mut buf).await?;

        // Deserialize
        serde_json::from_slice(&buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn write_request<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        // Serialize
        let data = serde_json::to_vec(&req)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Check max size
        if data.len() > Self::MAX_MESSAGE_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Message too large: {} bytes", data.len()),
            ));
        }

        // Write length prefix
        let len = data.len() as u32;
        io.write_all(&len.to_be_bytes()).await?;

        // Write message
        io.write_all(&data).await?;
        io.flush().await?;

        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
        res: Self::Response,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        // Serialize
        let data = serde_json::to_vec(&res)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Check max size
        if data.len() > Self::MAX_MESSAGE_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Message too large: {} bytes", data.len()),
            ));
        }

        // Write length prefix
        let len = data.len() as u32;
        io.write_all(&len.to_be_bytes()).await?;

        // Write message
        io.write_all(&data).await?;
        io.flush().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::io::Cursor;

    #[tokio::test]
    async fn test_request_roundtrip() {
        let mut codec = SyncCodec;
        let protocol = SYNC_PROTOCOL.to_string();

        let request = SyncRequest::GetStateDigest;

        // Write
        let mut buf = Vec::new();
        codec.write_request(&protocol, &mut buf, request.clone()).await.unwrap();

        // Read
        let mut cursor = Cursor::new(buf);
        let decoded = codec.read_request(&protocol, &mut cursor).await.unwrap();

        match decoded {
            SyncRequest::GetStateDigest => {},
            _ => panic!("Wrong variant"),
        }
    }

    #[tokio::test]
    async fn test_response_roundtrip() {
        let mut codec = SyncCodec;
        let protocol = SYNC_PROTOCOL.to_string();

        let response = SyncResponse::Error {
            message: "test".to_string(),
        };

        // Write
        let mut buf = Vec::new();
        codec.write_response(&protocol, &mut buf, response.clone()).await.unwrap();

        // Read
        let mut cursor = Cursor::new(buf);
        let decoded = codec.read_response(&protocol, &mut cursor).await.unwrap();

        match decoded {
            SyncResponse::Error { message } => {
                assert_eq!(message, "test");
            },
            _ => panic!("Wrong variant"),
        }
    }

    #[tokio::test]
    async fn test_message_size_limit() {
        let mut codec = SyncCodec;
        let protocol = SYNC_PROTOCOL.to_string();

        // Create a huge message
        let huge_records = vec![vec![0u8; SyncCodec::MAX_MESSAGE_SIZE]; 2];
        let request = SyncRequest::GetRecords {
            collection: "test".to_string(),
            keys: huge_records,
        };

        // Should fail to write
        let mut buf = Vec::new();
        let result = codec.write_request(&protocol, &mut buf, request).await;
        assert!(result.is_err());
    }
}
