use crate::Uri;
use http::uri::Parts;
use stepflow_base::{ObjectStoreFiltered, ObjectStoreContent};
use stepflow_data::{UriValue, Var, VarId, StateDataFiltered};
use super::{ActionResult, Step, Action, ActionId};
use crate::ActionError;


// NOTE: this is basically a hack
fn uri_join_relative(uri: Uri, relative_suffix: &str) -> Result<Uri, Box<dyn std::error::Error>> {
  let mut parts = Parts::from(uri);
  if let Some(path_and_query) = parts.path_and_query {
    let path_ends_with_slash = path_and_query.path().ends_with("/");
    let suffix_starts_with_slash = relative_suffix.starts_with("/");
    let new_path = match (path_ends_with_slash, suffix_starts_with_slash) {
      (false, false) => format!("{}/{}", path_and_query.path(), relative_suffix),
      (false, true) |
      (true, false) => format!("{}{}", path_and_query.path(), relative_suffix),
      (true, true) => {
        let mut path_without_ending_slash = path_and_query.path().to_owned();
        path_without_ending_slash.replace_range(path_without_ending_slash.len()-1.., "");
        path_without_ending_slash + relative_suffix
      }
    };
    parts.path_and_query = Some(new_path.parse()?);
  } else {
    parts.path_and_query = Some(relative_suffix.parse()?);
  }
  Ok(Uri::from_parts(parts)?)
}

#[derive(Debug)]
pub struct UrlStepAction {
  id: ActionId,
  base_url: Uri,
}

impl UrlStepAction {
  pub fn new(id: ActionId, base_url: Uri) -> Self {
    UrlStepAction {
      id,
      base_url,
    }
  }
}

impl Action for UrlStepAction {
  fn id(&self) -> &ActionId {
    &self.id
  }

  fn boxed(self) -> Box<dyn Action + Sync + Send> {
    Box::new(self)
  }

  fn start(&mut self, step: &Step, step_name: Option<&String>, _step_data: &StateDataFiltered, _vars: &ObjectStoreFiltered<Box<dyn Var + Send + Sync>, VarId>)
    -> Result<ActionResult, ActionError> {
      let path_str = match step_name {
        Some(name) => urlencoding::encode(&name[..]),
        None => step.id().to_string(),
      };
      let path = format!("/{}", path_str);
      let result_url = uri_join_relative(self.base_url.clone(), &path).map_err(|_e| ActionError::Other)?;
      let urival = UriValue::try_new(result_url.to_string()).map_err(|_e| ActionError::Other)?;
      Ok(ActionResult::StartWith(urival.boxed()))
    }
}

#[cfg(test)]
mod tests {
  use std::collections::HashSet;
  use super::{UrlStepAction, Uri, uri_join_relative};
  use stepflow_base::{ObjectStoreContent, ObjectStoreFiltered};
  use stepflow_data::{StateDataFiltered, UriValue};
  use stepflow_test_util::test_id;
  use super::super::{ActionResult, Action, ActionId, test_action_setup};

  #[test]
  fn uri_join() {
    let base_uri = "/hi".parse::<Uri>().unwrap();
    let base_uri_slash = "/hi/".parse::<Uri>().unwrap();

    assert_eq!(uri_join_relative(base_uri.clone(), "bye").unwrap().to_string(), "/hi/bye");
    assert_eq!(uri_join_relative(base_uri_slash.clone(), "bye").unwrap().to_string(), "/hi/bye");
    assert_eq!(uri_join_relative(base_uri.clone(), "/bye").unwrap().to_string(), "/hi/bye");
    assert_eq!(uri_join_relative(base_uri_slash.clone(), "/bye").unwrap().to_string(), "/hi/bye");
  }

  #[test]
  fn join() {
    let (step, state_data, varstore, _var_id, _val) = test_action_setup();
    let vars = ObjectStoreFiltered::new(&varstore, HashSet::new());
    let step_data_filtered = StateDataFiltered::new(&state_data, HashSet::new());

    let mut exec = UrlStepAction::new(test_id!(ActionId) ,"/test/url".parse().unwrap());
    let action_result = exec.start(&step, None, &step_data_filtered, &vars).unwrap();
    let uri = format!("/test/url/{}", step.id());
    let expected_val = UriValue::try_new(uri).unwrap();
    let expected_result = ActionResult::StartWith(expected_val.boxed());
    assert_eq!(action_result, expected_result);
  }

  #[test]
  fn encode_name() {
    let (step, state_data, varstore, _var_id, _val) = test_action_setup();
    let vars = ObjectStoreFiltered::new(&varstore, HashSet::new());
    let step_data_filtered = StateDataFiltered::new(&state_data, HashSet::new());

    let mut exec = UrlStepAction::new(test_id!(ActionId) ,"/test/url".parse().unwrap());
    let action_result = exec.start(&step, Some(&"/hi there?/".to_owned()), &step_data_filtered, &vars).unwrap();
    let expected_val = UriValue::try_new("/test/url/%2Fhi%20there%3F%2F".to_owned()).unwrap();
    let expected_result = ActionResult::StartWith(expected_val.boxed());
    assert_eq!(action_result, expected_result);
  }

}
