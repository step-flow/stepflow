use stepflow_base::{ObjectStoreContent, ObjectStoreFiltered, generate_id_type, IdError};
use stepflow_data::{StateData, StateDataFiltered, Value, Var, VarId};
use stepflow_step::{Step};
use crate::ActionError;

mod action_url;
pub use action_url::UrlStepAction;

mod action_htmlform;
pub use action_htmlform::{HtmlFormAction, HtmlFormConfig};

mod action_callback;
pub use action_callback::CallbackStepAction;

mod action_set_data;
pub use action_set_data::SetDataAction;


generate_id_type!(ActionId);

#[derive(Debug, Clone)]
pub enum ActionResult {
  StartWith(Box<dyn Value>),   // it's action's responsibility to advance step (Session.advance)
  Finished(StateData),            // caller of action advances the step with the output data... is this the Session?
  CannotFulfill,                  // could not fulfill the required output. ths isn't considered an error because it's ok to not always be able to fulfill right now
}

impl PartialEq for ActionResult {
    fn eq(&self, other: &Self) -> bool {
      match (self, other) {
        (ActionResult::StartWith(val), ActionResult::StartWith(val_other)) => {
          val == val_other
        },
        (ActionResult::Finished(data), ActionResult::Finished(data_other)) => {
          data == data_other
        },
        (ActionResult::CannotFulfill, ActionResult::CannotFulfill) => {
          true
        },
        (ActionResult::StartWith(_), _) |
        (ActionResult::Finished(_), _) |
        (ActionResult::CannotFulfill, _) => {
          false
        },
      }
    }
}

pub trait Action: std::fmt::Debug {
  fn id(&self) -> &ActionId;
  fn boxed(self) -> Box<dyn Action + Sync + Send>;
  fn start(&mut self, step: &Step, step_name: Option<&String>, step_data: &StateDataFiltered, vars: &ObjectStoreFiltered<Box<dyn Var + Send + Sync>, VarId>)
    -> Result<ActionResult, ActionError>;
}

impl ObjectStoreContent for Box<dyn Action + Sync + Send> {
    type IdType = ActionId;

    fn new_id(id_val: u32) -> Self::IdType {
      ActionId::new(id_val)
    }

    fn id(&self) -> &Self::IdType {
      self.as_ref().id()
    }
}

// setup for useful vars for testing.
#[cfg(test)]
pub fn test_action_setup<'a>() -> (Step, StateData, stepflow_base::ObjectStore<Box<dyn Var + Send + Sync>, VarId>, VarId, Box<dyn stepflow_data::Value>) {
  // setup var
  let mut varstore: stepflow_base::ObjectStore<Box<dyn Var + Send + Sync>, VarId> = stepflow_base::ObjectStore::new();
  let var_id = varstore.insert_new(None, |id| Ok(stepflow_data::StringVar::new(id).boxed())).unwrap();
  let var = varstore.get(&var_id).unwrap();

  // set a value in statedata
  let state_val = stepflow_data::StringValue::try_new("hi".to_owned()).unwrap().boxed();
  let mut state_data = StateData::new();
  state_data.insert(var, state_val.clone()).unwrap();

  let step = Step::new(stepflow_step::StepId::new(2), None, vec![]);
  (step, state_data, varstore, var_id, state_val)
}

#[cfg(test)]
mod tests {
  use std::convert::TryFrom;
  use stepflow_test_util::test_id;
  use stepflow_data::{StateData, TrueValue};
  use crate::{Action, ActionId};
  use super::{ActionResult, UrlStepAction};

  #[test]
  fn eq() {
    let result_start = ActionResult::StartWith(TrueValue::new().boxed());
    let result_finish = ActionResult::Finished(StateData::new());
    let result_cannot = ActionResult::CannotFulfill;

    assert_eq!(result_start, result_start);
    assert_ne!(result_start, result_finish);
    assert_ne!(result_start, result_cannot);

    assert_eq!(result_finish, result_finish);
    assert_ne!(result_finish, result_cannot);
  }

  #[test]
  fn object_store_content() {
    let test_action = UrlStepAction::new(test_id!(ActionId), http::Uri::try_from("test").unwrap());
    let test_action_id = test_action.id().clone();

    let boxed: Box<dyn Action> = test_action.boxed();
    let boxed_id = boxed.id().clone();
    
    assert_eq!(test_action_id, boxed_id);
  }

}