use stepflow_base::ObjectStoreFiltered;
use stepflow_data::{StateData, StateDataFiltered, var::{Var, VarId}, value::Value};
use stepflow_step::Step;
use stepflow_action::{Action, ActionId, ActionResult, ActionError};

#[derive(Debug)]
pub struct TestAction {
  id: ActionId,
  return_start_with: bool,
}

impl TestAction {
  pub fn new_with_id(id: ActionId, return_start_with: bool) -> Self {
    TestAction {
      id: id,
      return_start_with,
    }
  }
}

 impl Action for TestAction {
  fn id(&self) -> &ActionId {
    &self.id
  }

  fn boxed(self) -> Box<dyn Action + Sync + Send> {
    Box::new(self)
  }

  fn start(&mut self, _step: &Step, _step_name: Option<&String>, _step_data: &StateDataFiltered, _vars: &ObjectStoreFiltered<Box<dyn Var + Send + Sync>, VarId>)
      -> Result<ActionResult, ActionError> 
  {
    if self.return_start_with {
      let val: Box<dyn Value> = Box::new(stepflow_data::value::TrueValue::new());
      Ok(ActionResult::StartWith(val))
    } else {
      Ok(ActionResult::Finished(StateData::new()))
    }
  }
}