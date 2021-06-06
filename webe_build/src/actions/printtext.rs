use std::error::Error;

use crate::context::PlanContext;
use crate::action::Action;

pub struct PrintText;

impl Action for PrintText {
  // Prints the current context to the command line.
  fn perform(&self, context: &PlanContext) -> Result<PlanContext, Box<dyn Error>> {

    println!("ASDF1234");

    return Ok(context.clone());
  }
}