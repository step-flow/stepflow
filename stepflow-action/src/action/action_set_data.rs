use stepflow_base::ObjectStoreFiltered;
use stepflow_data::{StateDataFiltered, var::{Var, VarId}};
use super::{ActionResult, Action, ActionId, Step, StateData, ActionError};


/// Action that sets output data after a set number of attempts
#[derive(Debug)]
pub struct SetDataAction {
  id: ActionId,
  count: u64,
  after_attempt: u64,
  data: StateData,
}

impl SetDataAction {
  /// `data` is returned as [`ActionResult::Finished`] after `after_attempt` number of tries.
  /// If `after_attempt` is set to zero, it will set the data on the first call to [`start`](SetDataAction::start).
  pub fn new(id: ActionId, data: StateData, after_attempt: u64) -> Self {
    SetDataAction {
      id,
      count: 0,
      after_attempt,
      data,
    }
  }

  pub fn boxed(self) -> Box<dyn Action + Sync + Send> {
    Box::new(self)
  }
}

impl Action for SetDataAction {
  fn id(&self) -> &ActionId {
    &self.id
  }

  fn start(&mut self, _step: &Step, _step_name: Option<&str>, _step_data: &StateDataFiltered, _vars: &ObjectStoreFiltered<Box<dyn Var + Send + Sync>, VarId>)
    -> Result<ActionResult, ActionError>
  {
    if self.count >= self.after_attempt {
      Ok(ActionResult::Finished(self.data.clone()))
    } else {
      self.count += 1;
      Ok(ActionResult::CannotFulfill)
    }
  }
}



#[cfg(test)]
mod tests {
  use std::collections::HashSet;
  use stepflow_base::ObjectStoreFiltered;
  use stepflow_data::{StateData, StateDataFiltered};
  use stepflow_test_util::test_id;
  use crate::{ActionResult, Action, ActionId};
  use super::SetDataAction;
  use super::super::test_action_setup;

  #[test]
  fn on_attempts() {
    let (step, state_data, var_store, var_id, val) = test_action_setup();
    let mut allowed_ids = HashSet::new();
    allowed_ids.insert(var_id.clone());
    let vars = ObjectStoreFiltered::new(&var_store, allowed_ids);
    let step_data_filtered = StateDataFiltered::new(&state_data, HashSet::new());

    let mut expected_output = StateData::new();
    let var = vars.get(&var_id).unwrap();
    expected_output.insert(var, val.clone()).unwrap();

    let mut action_now = SetDataAction::new(
      test_id!(ActionId),
      state_data.clone(),
      0);
    assert!(matches!(
      action_now.start(&step, None, &step_data_filtered, &vars),
      Ok(ActionResult::Finished(output)) if output == expected_output));

    let mut action_after_3 = SetDataAction::new(
      test_id!(ActionId),
      state_data.clone(),
      3);
    for _ in 0..3 {
      assert_eq!(
        action_after_3.start(&step, None, &step_data_filtered, &vars),
        Ok(ActionResult::CannotFulfill));
    }
    assert!(matches!(
      action_after_3.start(&step, None, &step_data_filtered, &vars),
      Ok(ActionResult::Finished(output)) if output == expected_output));
  }
}
