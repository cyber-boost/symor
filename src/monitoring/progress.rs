use std::{
    collections::HashMap, path::PathBuf, sync::mpsc::{self, Receiver, Sender},
    time::{Duration, Instant, SystemTime},
};
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}
#[derive(Debug, Clone)]
pub struct ProgressEvent {
    pub operation_id: String,
    pub status: OperationStatus,
    pub progress: f32,
    pub message: String,
    pub timestamp: SystemTime,
}
#[derive(Debug, Clone)]
pub struct SyncOperation {
    pub id: String,
    pub path: PathBuf,
    pub operation_type: String,
    pub start_time: Instant,
    pub status: OperationStatus,
    pub progress: f32,
    pub total_items: usize,
    pub processed_items: usize,
}
pub struct ProgressTracker {
    operations: HashMap<String, SyncOperation>,
    event_sender: Sender<ProgressEvent>,
    event_receiver: Receiver<ProgressEvent>,
    start_time: Instant,
}
impl ProgressTracker {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            operations: HashMap::new(),
            event_sender: sender,
            event_receiver: receiver,
            start_time: Instant::now(),
        }
    }
    pub fn start_operation(
        &mut self,
        id: String,
        path: PathBuf,
        operation_type: String,
    ) -> Result<(), String> {
        if self.operations.contains_key(&id) {
            return Err(format!("Operation {} already exists", id));
        }
        let operation = SyncOperation {
            id: id.clone(),
            path,
            operation_type,
            start_time: Instant::now(),
            status: OperationStatus::Running,
            progress: 0.0,
            total_items: 0,
            processed_items: 0,
        };
        self.operations.insert(id.clone(), operation);
        let event = ProgressEvent {
            operation_id: id,
            status: OperationStatus::Running,
            progress: 0.0,
            message: "Operation started".to_string(),
            timestamp: SystemTime::now(),
        };
        let _ = self.event_sender.send(event);
        Ok(())
    }
    pub fn update_progress(
        &mut self,
        id: &str,
        progress: f32,
        message: String,
    ) -> Result<(), String> {
        if let Some(operation) = self.operations.get_mut(id) {
            operation.progress = progress.clamp(0.0, 1.0);
            let event = ProgressEvent {
                operation_id: id.to_string(),
                status: operation.status.clone(),
                progress,
                message,
                timestamp: SystemTime::now(),
            };
            let _ = self.event_sender.send(event);
            Ok(())
        } else {
            Err(format!("Operation {} not found", id))
        }
    }
    pub fn complete_operation(&mut self, id: &str) -> Result<(), String> {
        if let Some(operation) = self.operations.get_mut(id) {
            operation.status = OperationStatus::Completed;
            operation.progress = 1.0;
            let event = ProgressEvent {
                operation_id: id.to_string(),
                status: OperationStatus::Completed,
                progress: 1.0,
                message: "Operation completed".to_string(),
                timestamp: SystemTime::now(),
            };
            let _ = self.event_sender.send(event);
            Ok(())
        } else {
            Err(format!("Operation {} not found", id))
        }
    }
    pub fn fail_operation(&mut self, id: &str, error: String) -> Result<(), String> {
        if let Some(operation) = self.operations.get_mut(id) {
            operation.status = OperationStatus::Failed;
            let event = ProgressEvent {
                operation_id: id.to_string(),
                status: OperationStatus::Failed,
                progress: operation.progress,
                message: error,
                timestamp: SystemTime::now(),
            };
            let _ = self.event_sender.send(event);
            Ok(())
        } else {
            Err(format!("Operation {} not found", id))
        }
    }
    pub fn get_operation(&self, id: &str) -> Option<&SyncOperation> {
        self.operations.get(id)
    }
    pub fn get_all_operations(&self) -> Vec<&SyncOperation> {
        self.operations.values().collect()
    }
    pub fn get_stats(&self) -> ProgressStats {
        let total_operations = self.operations.len();
        let running_operations = self
            .operations
            .values()
            .filter(|op| op.status == OperationStatus::Running)
            .count();
        let completed_operations = self
            .operations
            .values()
            .filter(|op| op.status == OperationStatus::Completed)
            .count();
        let failed_operations = self
            .operations
            .values()
            .filter(|op| op.status == OperationStatus::Failed)
            .count();
        ProgressStats {
            total_operations,
            running_operations,
            completed_operations,
            failed_operations,
            uptime: self.start_time.elapsed(),
        }
    }
    pub fn receive_event(&self) -> Result<ProgressEvent, mpsc::TryRecvError> {
        self.event_receiver.try_recv()
    }
}
#[derive(Debug, Clone)]
pub struct ProgressStats {
    pub total_operations: usize,
    pub running_operations: usize,
    pub completed_operations: usize,
    pub failed_operations: usize,
    pub uptime: Duration,
}