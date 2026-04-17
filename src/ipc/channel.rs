use super::{IpcError, Result};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMessage {
    pub id: String,
    pub source: String,
    pub target: String,
    pub payload: Vec<u8>,
    pub timestamp: u64,
    pub ttl: Option<u32>,
    pub priority: MessagePriority,
    pub message_type: MessageType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessagePriority {
    Low,
    Normal,
    High,
    Critical,
}

impl Default for MessagePriority {
    fn default() -> Self {
        MessagePriority::Normal
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    Request,
    Response,
    Event,
    Command,
    Heartbeat,
}

impl Default for MessageType {
    fn default() -> Self {
        MessageType::Event
    }
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
            ttl: None,
            priority: MessagePriority::default(),
            message_type: MessageType::default(),
        }
    }

    pub fn with_ttl(mut self, ttl: u32) -> Self {
        self.ttl = Some(ttl);
        self
    }

    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_type(mut self, message_type: MessageType) -> Self {
        self.message_type = message_type;
        self
    }

    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            current_time > self.timestamp + (ttl as u64)
        } else {
            false
        }
    }
}

pub struct ZeroCopyMessage {
    pub id: String,
    pub source: String,
    pub target: String,
    pub payload: Bytes,
    pub timestamp: u64,
    pub ttl: Option<u32>,
    pub priority: MessagePriority,
    pub message_type: MessageType,
}

impl ZeroCopyMessage {
    pub fn new(source: impl Into<String>, target: impl Into<String>, payload: Bytes) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            source: source.into(),
            target: target.into(),
            payload,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ttl: None,
            priority: MessagePriority::default(),
            message_type: MessageType::default(),
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            current_time > self.timestamp + (ttl as u64)
        } else {
            false
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
        self.sender
            .send(message)
            .map_err(|e| IpcError::ChannelError(format!("Failed to send message: {}", e)))?;
        Ok(())
    }

    pub fn receive(&self) -> Result<IpcMessage> {
        let receiver = self
            .receiver
            .lock()
            .map_err(|e| IpcError::ChannelError(format!("Lock poisoned: {}", e)))?;
        receiver
            .recv()
            .map_err(|e| IpcError::ChannelError(format!("Failed to receive message: {}", e)))
    }

    pub fn try_receive(&self) -> Result<Option<IpcMessage>> {
        let receiver = self
            .receiver
            .lock()
            .map_err(|e| IpcError::ChannelError(format!("Lock poisoned: {}", e)))?;
        match receiver.try_recv() {
            Ok(msg) => Ok(Some(msg)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(e) => Err(IpcError::ChannelError(format!(
                "Failed to receive message: {}",
                e
            ))),
        }
    }

    pub fn receive_with_timeout(&self, timeout: Duration) -> Result<Option<IpcMessage>> {
        let receiver = self
            .receiver
            .lock()
            .map_err(|e| IpcError::ChannelError(format!("Lock poisoned: {}", e)))?;
        match receiver.recv_timeout(timeout) {
            Ok(msg) => Ok(Some(msg)),
            Err(mpsc::RecvTimeoutError::Timeout) => Ok(None),
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                Err(IpcError::ChannelError("Channel disconnected".to_string()))
            }
        }
    }
}

impl Default for IpcChannel {
    fn default() -> Self {
        Self::new()
    }
}

pub struct BroadcastChannel {
    sender: Arc<RwLock<mpsc::Sender<IpcMessage>>>,
    receivers: Arc<RwLock<HashMap<String, mpsc::Receiver<IpcMessage>>>>,
}

impl BroadcastChannel {
    pub fn new() -> Self {
        Self {
            sender: Arc::new(RwLock::new(mpsc::channel().0)),
            receivers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add_receiver(&mut self, name: impl Into<String>) -> mpsc::Receiver<IpcMessage> {
        let (sender, receiver) = mpsc::channel();
        self.receivers
            .write()
            .unwrap()
            .insert(name.into(), receiver);
        sender
    }

    pub fn broadcast(&self, message: IpcMessage) -> Result<()> {
        let sender = self.sender.read().unwrap();
        for (_, receiver) in self.receivers.read().unwrap().iter() {
            if let Err(e) = receiver.try_recv() {
                if let mpsc::TryRecvError::Disconnected = e {
                    continue;
                }
            }
        }
        sender
            .send(message)
            .map_err(|e| IpcError::ChannelError(format!("Failed to broadcast message: {}", e)))?;
        Ok(())
    }
}

impl Default for BroadcastChannel {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ChannelManager {
    channels: Arc<RwLock<HashMap<String, IpcChannel>>>,
    broadcast_channels: Arc<RwLock<HashMap<String, BroadcastChannel>>>,
}

impl ChannelManager {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            broadcast_channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn create_channel(&self, name: impl Into<String>) -> Result<()> {
        let name = name.into();
        let mut channels = self.channels.write().map_err(|e| {
            IpcError::ChannelError(format!("Failed to acquire lock: {}", e))
        })?;
        if channels.contains_key(&name) {
            return Err(IpcError::ChannelError(format!(
                "Channel '{}' already exists",
                name
            )));
        }
        channels.insert(name, IpcChannel::new());
        Ok(())
    }

    pub fn get_channel(&self, name: &str) -> Result<IpcChannel> {
        let channels = self.channels.read().map_err(|e| {
            IpcError::ChannelError(format!("Failed to acquire lock: {}", e))
        })?;
        channels
            .get(name)
            .cloned()
            .ok_or_else(|| IpcError::ChannelError(format!("Channel '{}' not found", name)))
    }

    pub fn remove_channel(&self, name: &str) -> Result<()> {
        let mut channels = self.channels.write().map_err(|e| {
            IpcError::ChannelError(format!("Failed to acquire lock: {}", e))
        })?;
        channels
            .remove(name)
            .ok_or_else(|| IpcError::ChannelError(format!("Channel '{}' not found", name)))?;
        Ok(())
    }

    pub fn create_broadcast_channel(&self, name: impl Into<String>) -> Result<()> {
        let name = name.into();
        let mut broadcast_channels = self.broadcast_channels.write().map_err(|e| {
            IpcError::ChannelError(format!("Failed to acquire lock: {}", e))
        })?;
        if broadcast_channels.contains_key(&name) {
            return Err(IpcError::ChannelError(format!(
                "Broadcast channel '{}' already exists",
                name
            )));
        }
        broadcast_channels.insert(name, BroadcastChannel::new());
        Ok(())
    }

    pub fn list_channels(&self) -> Vec<String> {
        self.channels.read().unwrap().keys().cloned().collect()
    }

    pub fn list_broadcast_channels(&self) -> Vec<String> {
        self.broadcast_channels
            .read()
            .unwrap()
            .keys()
            .cloned()
            .collect()
    }
}

impl Default for ChannelManager {
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

    #[test]
    fn test_message_priority() {
        let message = IpcMessage::new("source", "target", vec![1, 2, 3])
            .with_priority(MessagePriority::High);
        assert_eq!(message.priority, MessagePriority::High);
    }

    #[test]
    fn test_message_type() {
        let message = IpcMessage::new("source", "target", vec![1, 2, 3])
            .with_type(MessageType::Request);
        assert_eq!(message.message_type, MessageType::Request);
    }

    #[test]
    fn test_message_ttl() {
        let message = IpcMessage::new("source", "target", vec![1, 2, 3]).with_ttl(60);
        assert!(!message.is_expired());
    }

    #[test]
    fn test_broadcast_channel() {
        let mut broadcast = BroadcastChannel::new();
        let _receiver = broadcast.add_receiver("test_receiver");

        let message = IpcMessage::new("source", "broadcast", vec![1, 2, 3]);
        assert!(broadcast.broadcast(message).is_ok());
    }

    #[test]
    fn test_channel_manager() {
        let manager = ChannelManager::new();

        assert!(manager.create_channel("test_channel").is_ok());
        assert!(manager.create_channel("test_channel").is_err());

        let channel = manager.get_channel("test_channel");
        assert!(channel.is_ok());

        assert!(manager.remove_channel("test_channel").is_ok());
        assert!(manager.remove_channel("nonexistent").is_err());
    }

    #[test]
    fn test_channel_manager_broadcast() {
        let manager = ChannelManager::new();

        assert!(manager.create_broadcast_channel("test_broadcast").is_ok());
        assert!(manager.create_broadcast_channel("test_broadcast").is_err());

        let broadcasts = manager.list_broadcast_channels();
        assert!(broadcasts.contains(&"test_broadcast".to_string()));
    }

    #[test]
    fn test_receive_with_timeout() {
        let channel = IpcChannel::new();

        let result = channel.receive_with_timeout(Duration::from_millis(100));
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
