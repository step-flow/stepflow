use std::{sync::RwLock, unimplemented};
use stepflow_base::ObjectStoreFiltered;
use stepflow_data::{StateDataFiltered, var::{Var, VarId}};
use super::{ActionResult, Action, ActionId, Step};
use crate::ActionError;


/// Action that wraps a closure.
pub struct CallbackStepAction<F> {
  id: ActionId,
  cb: RwLock<F>,  // it'd be nice to someday not use interior mutability here
}

impl<F> std::fmt::Debug for CallbackStepAction<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "CallbackStepAction({})", self.id)
    }
}

impl<F> CallbackStepAction<F> 
    where F: FnMut(&Step, Option<&str>, &StateDataFiltered, &ObjectStoreFiltered<Box<dyn Var + Send + Sync>, VarId>) -> Result<ActionResult, ActionError> + Send + Sync
{
  pub fn new(id: ActionId, cb: F) -> Self {
    CallbackStepAction {
      id,
      cb: RwLock::new(cb),
    }
  }
}

impl<F> Action for CallbackStepAction<F>
    where F: FnMut(&Step, Option<&str>, &StateDataFiltered, &ObjectStoreFiltered<Box<dyn Var + Send + Sync>, VarId>) -> Result<ActionResult, ActionError> + Send + Sync
{
  fn id(&self) -> &ActionId {
    &self.id
  }

  fn boxed(self) -> Box<dyn Action + Sync + Send> {
    unimplemented!();
//    Box::new(self)
  }

  fn start(&mut self, step: &Step, step_name: Option<&str>, step_data: &StateDataFiltered, vars: &ObjectStoreFiltered<Box<dyn Var + Send + Sync>, VarId>)
     -> Result<ActionResult, ActionError> {
    let mut cb = self.cb.try_write().map_err(|_e| ActionError::Other)?;
    cb(step, step_name, step_data, vars)
  }
}


#[cfg(test)]
mod tests {
  use std::collections::HashSet;
  use stepflow_base::ObjectStoreFiltered;
  use stepflow_data::StateDataFiltered;
  use stepflow_test_util::test_id;
  use crate::{ Action, ActionId, ActionResult};
  use super::CallbackStepAction;
  use super::super::test_action_setup;

  #[test]
  fn basic_callback() {
    let mut count = 0;

    {
      let mut exec = CallbackStepAction::new(test_id!(ActionId),
      |_, _, _, _| {
        count += 1;
        Ok(ActionResult::CannotFulfill)
      });

      let (step, state_data, var_store, _var_id, _val) = test_action_setup();
      let vars = ObjectStoreFiltered::new(&var_store, HashSet::new());
      let step_data_filtered = StateDataFiltered::new(&state_data, HashSet::new());

      let start_action1 = exec.start(&step, None, &step_data_filtered, &vars);
      assert_eq!(start_action1, Ok(ActionResult::CannotFulfill));

      let start_action2 = exec.start(&step, None, &step_data_filtered, &vars);
      assert_eq!(start_action2, Ok(ActionResult::CannotFulfill));

      let start_action3 = exec.start(&step, None, &step_data_filtered, &vars);
      assert_eq!(start_action3, Ok(ActionResult::CannotFulfill));
    }
    assert_eq!(count, 3);
  }
}
