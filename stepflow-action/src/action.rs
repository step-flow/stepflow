use stepflow_base::{ObjectStoreContent, ObjectStoreFiltered, generate_id_type, IdError};
use stepflow_data::{StateData, StateDataFiltered, value::Value, var::{Var, VarId}};
use stepflow_step::{Step};
use crate::ActionError;

mod action_string_template;
pub use action_string_template::StringTemplateAction;

mod action_htmlform;
pub use action_htmlform::{HtmlFormAction, HtmlFormConfig};

mod action_set_data;
pub use action_set_data::SetDataAction;

generate_id_type!(ActionId);

/// The result of [`Action::start()`]
#[derive(Debug, Clone)]
pub enum ActionResult {
  /// The action requires the caller to fulfill the [`Step`](stepflow_step::Step)'s outputs.
  /// The value's meaning is [`Action`] dependent.
  /// When the caller obtains the output data (i.e. with a form), it can then advance the `Session`.
  /// ```
  /// # use stepflow_action::ActionResult;
  /// # use stepflow_data::value::StringValue;
  /// # fn respond_with_redirect(uri: &StringValue) {}
  /// # let action_result = ActionResult::StartWith(StringValue::try_new("name-form").unwrap().boxed());
  /// if let ActionResult::StartWith(uri) = action_result {
  ///   respond_with_redirect(uri.downcast::<StringValue>().unwrap())
  /// }
  /// ```
  StartWith(Box<dyn Value>),

  /// The action fulfilled the ouputs with the results in the [`StateData`].
  Finished(StateData),

  /// The action was not able to fulfill the ouputs as a result of a normal condition
  /// such as a minimum time duration. This should not be used for error situations.
  CannotFulfill,
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

/// `Action`s fulfill the outputs of a [`Step`]
pub trait Action: std::fmt::Debug + stepflow_base::as_any::AsAny {
  /// Get the ID for the Action
  fn id(&self) -> &ActionId;

  /// Start the action for a [`Step`]
  ///
  /// `step_data` and `vars` only have access to input and output data declared by the Step.
  fn start(&mut self, step: &Step, step_name: Option<&str>, step_data: &StateDataFiltered, vars: &ObjectStoreFiltered<Box<dyn Var + Send + Sync>, VarId>)
    -> Result<ActionResult, ActionError>;
}

// implement downcast helpers that have trait bounds to make it a little safer
impl dyn Action + Send + Sync {
  pub fn downcast<T>(&self) -> Option<&T>
    where T: Action + std::any::Any
  {
    self.as_any().downcast_ref::<T>()
  }
  pub fn is<T>(&self) -> bool 
    where T: Action + std::any::Any
  {
    self.as_any().is::<T>()
  }
}

impl ObjectStoreContent for Box<dyn Action + Sync + Send> {
    type IdType = ActionId;

    fn new_id(id_val: u16) -> Self::IdType {
      ActionId::new(id_val)
    }

    fn id(&self) -> &Self::IdType {
      self.as_ref().id()
    }
}

// setup for useful vars for testing.
#[cfg(test)]
pub fn test_action_setup<'a>() -> (Step, StateData, stepflow_base::ObjectStore<Box<dyn Var + Send + Sync>, VarId>, VarId, Box<dyn stepflow_data::value::Value>) {
  // setup var
  let mut var_store: stepflow_base::ObjectStore<Box<dyn Var + Send + Sync>, VarId> = stepflow_base::ObjectStore::new();
  let var_id = var_store.insert_new(|id| Ok(stepflow_data::var::StringVar::new(id).boxed())).unwrap();
  let var = var_store.get(&var_id).unwrap();

  // set a value in statedata
  let state_val = stepflow_data::value::StringValue::try_new("hi").unwrap().boxed();
  let mut state_data = StateData::new();
  state_data.insert(var, state_val.clone()).unwrap();

  let step = Step::new(stepflow_step::StepId::new(2), None, vec![]);
  (step, state_data, var_store, var_id, state_val)
}

#[cfg(test)]
mod tests {
  use stepflow_test_util::test_id;
  use stepflow_data::{StateData, value::TrueValue};
  use super::{ActionId, HtmlFormAction, SetDataAction, ActionResult};

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
  fn downcast() {
    let action = HtmlFormAction::new(test_id!(ActionId), Default::default()).boxed();
    assert!(action.is::<HtmlFormAction>());
    assert!(!action.is::<SetDataAction>());
  }
}