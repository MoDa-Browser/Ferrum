use super::{IpcError, Result};
use serde::{Deserialize, Serialize};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMessage {
    pub id: String,
    pub source: String,
    pub target: String,
    pub payload: Vec<u8>,
    pub timestamp: u64,
}

impl IpcMessage {
    pub fn new(source: impl Into<String>, target: impl Into<String>, payload: Vec<u8>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            source: source.into(),
            target: target.into(),
            payload,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

pub struct IpcChannel {
    sender: mpsc::Sender<IpcMessage>,
    receiver: Arc<Mutex<mpsc::Receiver<IpcMessage>>>,
}

impl IpcChannel {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    pub fn send(&self, message: IpcMessage) -> Result<()> {
        self.sender.send(message).map_err(|e| {
            IpcError::ChannelError(format!("Failed to send message: {}", e))
        })?;
        Ok(())
    }

    pub fn receive(&self) -> Result<IpcMessage> {
        let receiver = self.receiver.lock()
            .map_err(|e| IpcError::ChannelError(format!("Lock poisoned: {}", e)))?;
        receiver.recv().map_err(|e| {
            IpcError::ChannelError(format!("Failed to receive message: {}", e))
        })
    }

    pub fn try_receive(&self) -> Result<Option<IpcMessage>> {
        let receiver = self.receiver.lock()
            .map_err(|e| IpcError::ChannelError(format!("Lock poisoned: {}", e)))?;
        match receiver.try_recv() {
            Ok(msg) => Ok(Some(msg)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(e) => Err(IpcError::ChannelError(format!("Failed to receive message: {}", e))),
        }
    }
}

impl Default for IpcChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_channel() {
        let channel = IpcChannel::new();
        
        let message = IpcMessage::new("source", "target", vec![1, 2, 3]);
        assert!(channel.send(message.clone()).is_ok());
        
        let received = channel.receive().unwrap();
        assert_eq!(received.source, "source");
        assert_eq!(received.target, "target");
    }
}
