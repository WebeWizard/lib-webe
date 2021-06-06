use std::time::{Instant};

use crate::plan::Plan;
use crate::taskhistory::TaskHistory;

pub enum PlanHistoryStatus {
  InProgress,
  Success,
  Error
}

pub struct PlanHistory {
  id: webe_id::WebeID,
  plan: Plan,
  start: Instant,
  end: Instant,
  status: PlanHistoryStatus,
  task_history: Vec<TaskHistory>
}