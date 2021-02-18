use std::collections::HashMap;

use stepflow_base::{ObjectStoreFiltered, ObjectStoreContent};
use stepflow_data::{StateDataFiltered, value::StringValue, var::{Var, VarId}};
use super::{ActionResult, Step, Action, ActionId};
use crate::{render_template, EscapedString};
use crate::ActionError;



#[derive(Debug)]
pub struct StringTemplateAction<T> {
  id: ActionId,
  template_escaped: T,
}

impl<T> StringTemplateAction<T> 
    where T: EscapedString
{
  /// Create a new instance.
  ///
  /// `template_escaped` must already be escaped. Parameters accepted within is `{{step}}`.
  /// If the [`Step`] has a name, that will be populated. If not, it will be the [`StepId`].
  pub fn new(id: ActionId, template_escaped: T) -> Self {
    StringTemplateAction {
      id,
      template_escaped,
    }
  }
}

impl<T> Action for StringTemplateAction<T> 
    where T: EscapedString
{
  fn id(&self) -> &ActionId {
    &self.id
  }

  fn boxed(self) -> Box<dyn Action + Sync + Send> {
    Box::new(self)
  }

  fn start(&mut self, step: &Step, step_name: Option<&str>, _step_data: &StateDataFiltered, _vars: &ObjectStoreFiltered<Box<dyn Var + Send + Sync>, VarId>)
      -> Result<ActionResult, ActionError> 
  {
    let escaped_step = match step_name {
      Some(name) => T::from_unescaped(name),
      None => T::from_unescaped(&step.id().to_string()[..]),
    };

    let mut params: HashMap<&str, T> = HashMap::new();
    params.insert("step", escaped_step);

    let result_str = render_template::<T>(&self.template_escaped, params);
    let string_val = StringValue::try_new(result_str).map_err(|_e| ActionError::Other)?;
    Ok(ActionResult::StartWith(string_val.boxed()))
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashSet;
  use super::{StringTemplateAction};
  use stepflow_base::{ObjectStoreContent, ObjectStoreFiltered};
  use stepflow_data::{StateDataFiltered, value::{StringValue}};
  use stepflow_test_util::test_id;
  use super::super::{ActionResult, Action, ActionId, test_action_setup};
  use crate::{EscapedString, UriEscapedString};


  #[test]
  fn basic() {
    let (step, state_data, var_store, _var_id, _val) = test_action_setup();
    let vars = ObjectStoreFiltered::new(&var_store, HashSet::new());
    let step_data_filtered = StateDataFiltered::new(&state_data, HashSet::new());

    let mut exec = StringTemplateAction::new(test_id!(ActionId) ,UriEscapedString::already_escaped("/test/{{step}}/uri#{{step}}".to_owned()));
    let action_result = exec.start(&step, None, &step_data_filtered, &vars).unwrap();
    let uri = format!("/test/{}/uri#{}", step.id(), step.id());
    let expected_val = StringValue::try_new(uri).unwrap();
    let expected_result = ActionResult::StartWith(expected_val.boxed());
    assert_eq!(action_result, expected_result);
  }

  #[test]
  fn encode_name() {
    let (step, state_data, var_store, _var_id, _val) = test_action_setup();
    let vars = ObjectStoreFiltered::new(&var_store, HashSet::new());
    let step_data_filtered = StateDataFiltered::new(&state_data, HashSet::new());

    let mut exec = StringTemplateAction::new(test_id!(ActionId) ,UriEscapedString::already_escaped("/test/uri/{{step}}".to_owned()));
    let action_result = exec.start(&step, Some("/hi there?/"), &step_data_filtered, &vars).unwrap();
    let expected_val = StringValue::try_new("/test/uri/%2Fhi%20there%3F%2F").unwrap();
    let expected_result = ActionResult::StartWith(expected_val.boxed());
    println!("ACTION: {:?}", action_result);
    assert_eq!(action_result, expected_result);
  }

}
