use std::time::Instant;
use std::future::Future;
use std::task::Context;
use std::task::Poll;
use std::pin::Pin;

use crate::context::PlanContext;

pub trait Action {
  fn perform(&self, context: &PlanContext) -> ActionHistory;
}


pub enum ActionHistoryStatus {
  InProgress,
  Success,
  Error
}

pub struct ActionHistory {
  id: webe_id::WebeID,
  start: Instant,
  end: Instant,
  status:  ActionHistoryStatus
}

impl ActionHistory {
  pub fn new() -> ActionHistory{
    ActionHistory {}
  }
}

impl Future for ActionHistory {
  type Output = Vec<u8>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {

  }
}