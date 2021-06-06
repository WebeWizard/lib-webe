use std::error::Error;

use crate::context::PlanContext;
use crate::action::Action;

pub struct PrintPlanContextAction;

impl Action for PrintPlanContextAction {
  // Prints the current context to the command line.
  fn perform(&self, context: &PlanContext) -> Result<PlanContext, Box<dyn Error>> {
    return Ok(context.clone());
  }
}