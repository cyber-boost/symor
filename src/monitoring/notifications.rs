use anyhow::Result;
use std::{
    path::Path, sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Success,
}
#[derive(Debug, Clone)]
pub struct FileChangeNotification {
    pub path: std::path::PathBuf,
    pub change_type: String,
    pub timestamp: std::time::SystemTime,
    pub level: NotificationLevel,
}
pub struct NotificationSystem {
    sender: Sender<FileChangeNotification>,
    receiver: Receiver<FileChangeNotification>,
    subscribers: Vec<Box<dyn ChangeSubscriber>>,
    enabled: bool,
}
impl NotificationSystem {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver,
            subscribers: Vec::new(),
            enabled: true,
        }
    }
    pub fn subscribe(&mut self, subscriber: Box<dyn ChangeSubscriber>) {
        self.subscribers.push(subscriber);
    }
    pub fn notify_file_change(
        &self,
        notification: FileChangeNotification,
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        let _ = self.sender.send(notification.clone());
        for subscriber in &self.subscribers {
            subscriber.on_file_change(&notification);
        }
        Ok(())
    }
    pub fn notify_sync_complete(&self, path: &Path, duration: Duration) -> Result<()> {
        for subscriber in &self.subscribers {
            subscriber.on_sync_complete(path, duration);
        }
        Ok(())
    }
    pub fn notify_error(&self, error: &anyhow::Error) -> Result<()> {
        for subscriber in &self.subscribers {
            subscriber.on_error(error);
        }
        Ok(())
    }
    pub fn receive_notification(&self) -> Result<Option<FileChangeNotification>> {
        match self.receiver.try_recv() {
            Ok(notification) => Ok(Some(notification)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(mpsc::TryRecvError::Disconnected) => {
                Err(anyhow::anyhow!("Notification channel disconnected"))
            }
        }
    }
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}
pub trait ChangeSubscriber: Send + Sync {
    fn on_file_change(&self, notification: &FileChangeNotification);
    fn on_sync_complete(&self, path: &Path, duration: Duration);
    fn on_error(&self, error: &anyhow::Error);
}
pub struct ConsoleSubscriber;
impl ChangeSubscriber for ConsoleSubscriber {
    fn on_file_change(&self, notification: &FileChangeNotification) {
        let level_str = match notification.level {
            NotificationLevel::Info => "INFO",
            NotificationLevel::Warning => "WARN",
            NotificationLevel::Error => "ERROR",
            NotificationLevel::Success => "SUCCESS",
        };
        println!(
            "[{}] {}: {:?}", level_str, notification.change_type, notification.path
        );
    }
    fn on_sync_complete(&self, path: &Path, duration: Duration) {
        println!("Sync completed for {:?} in {:.2}ms", path, duration.as_millis());
    }
    fn on_error(&self, error: &anyhow::Error) {
        eprintln!("Error: {}", error);
    }
}