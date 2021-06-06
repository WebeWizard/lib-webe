use std::error::Error;

use crate::context::PlanContext;
use crate::action::Action;
use crate::taskhistory::TaskHistory;

pub enum TaskExecMode {
  Serial,
  Asynchronous
}

// pub enum TaskFailMode {
//   FailPlan, // error and fail the entire plan
//   FailTask, // error and fail the parent task
//   RetryThenFailPlan(u8), // 
//   RetryThenFailTask(u8)
// }

pub struct Task {
  name: String,
  description: String,
  exec_mode: TaskExecMode,
  //fail_mode: TaskFailMode,
  actions: Vec::<Box<dyn Action>>
}

impl Action for Task {
  fn perform(&self, context: &PlanContext) -> TaskHistory {
    let mut taskhistory = TaskHistory::new();
    let new_context = context.clone();
    match &self.exec_mode {
      Serial => {
        for action in self.actions.iter() {
          action.perform(&new_context);
        }
      }
      Asynchronous => {}
    }
    return Ok(context.clone());
  }
}