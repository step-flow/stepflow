use std::fmt::Write;
use stepflow_base::ObjectStoreFiltered;
use stepflow_data::{StateDataFiltered, Var, VarId, StringValue, StringVar, UriVar, EmailVar, BoolVar};
use super::{ActionResult, Action, ActionId, Step, ActionError};


struct HtmlEscapedString(String);
impl HtmlEscapedString {
  pub fn new(unescaped_str: &str) -> Self {
    HtmlEscapedString(htmlescape::encode_attribute(unescaped_str))
  }
}


// Someday we should have a HtmlFormTag trait that any var can implement and then call that for their tag. not able until we can cast a Var trait to a HtmlFormTag trait
#[derive(Debug)]
pub struct HtmlFormConfig {
  pub stringvar_html_template: String,
  pub urivar_html_template: String,
  pub emailvar_html_template: String,
  pub boolvar_html_template: String,

  pub prefix_html_template: Option<String>, // ie. label before each input field
  pub wrap_tag: Option<String>, // ie. wrap entire element in a <div></div>
}

impl HtmlFormConfig {
  fn format_html_template(tag_template: &String, name_escaped: &HtmlEscapedString) -> String {
    tag_template.replace("{{name}}", &name_escaped.0[..])
  }

  fn valid_wraptag(&self) -> Option<&String> {
    if let Some(wrap_tag) = &self.wrap_tag {
      if !wrap_tag.is_empty() {
        return Some(wrap_tag);
      }
    }
    None
  }

  fn format_input_template(&self, html_template: &String, name_escaped: &HtmlEscapedString) -> Result<String, std::fmt::Error> {
    let mut html = String::with_capacity(html_template.len() + name_escaped.0.len()); // rough guss

    // write the head of the wrap
    if let Some(wrap_tag) = self.valid_wraptag() {
      if !wrap_tag.is_empty() {
        write!(html, "<{}>", wrap_tag)?;
      }
    }

    // write the prefix
    if let Some(prefix_html_template) = &self.prefix_html_template {
      let prefix_html = Self::format_html_template(prefix_html_template, name_escaped);
      html.write_str(&prefix_html[..])?;
    }

    // write the tag
    let input_html = Self::format_html_template(&html_template, name_escaped);
    html.write_str(&input_html[..])?;

    // write the tail of the wrap
    if let Some(wrap_tag) = self.valid_wraptag() {
      write!(html, "</{}>", wrap_tag)?;
    }
  

    Ok(html)
  }
}

impl Default for HtmlFormConfig {
    fn default() -> Self {
        HtmlFormConfig {
          stringvar_html_template: "<input name='{{name}}' />".to_owned(),
          urivar_html_template: "<input name='{{name}}' type='url' />".to_owned(),
          emailvar_html_template: "<input name='{{name}}' type='email' />".to_owned(),
          boolvar_html_template: "<input name='{{name}}' type='checkbox' />".to_owned(),
          prefix_html_template: None,
          wrap_tag: None,
        }
    }
}


#[derive(Debug)]
pub struct HtmlFormAction {
  id: ActionId,
  html_config: HtmlFormConfig,
}

impl HtmlFormAction {
  pub fn new(id: ActionId, html_config: HtmlFormConfig) -> Self {
    HtmlFormAction {
      id,
      html_config,
    }
  }
}

impl Action for HtmlFormAction {
  fn id(&self) -> &ActionId {
    &self.id
  }

  fn boxed(self) -> Box<dyn Action + Sync + Send> {
    Box::new(self)
  }

  fn start(&mut self, step: &Step, _step_name: Option<&String>, _step_data: &StateDataFiltered, vars: &ObjectStoreFiltered<Box<dyn Var + Send + Sync>, VarId>)
    -> Result<ActionResult, ActionError>
  {
    const AVG_NAME_LEN: usize = 5;
    let mut html = String::with_capacity(step.get_output_vars().len() * (self.html_config.stringvar_html_template.len() + AVG_NAME_LEN));
    for var_id in step.get_output_vars().iter() {
      let name = vars.name_from_id(var_id).ok_or_else(|| ActionError::Other)?;
      let name_escaped = HtmlEscapedString::new(name);

      let var = vars.get(var_id).ok_or_else(|| ActionError::VarInvalid(var_id.clone()))?;
      let html_template;
      if var.is::<StringVar>() {
        html_template = &self.html_config.stringvar_html_template;
      } else if var.is::<UriVar>() {
        html_template = &self.html_config.urivar_html_template;
      } else if var.is::<EmailVar>() {
        html_template = &self.html_config.emailvar_html_template;
      } else if var.is::<BoolVar>() {
        html_template = &self.html_config.boolvar_html_template;
      } else {
        // perhaps panic when in debug? 
        // maybe in the future we should ask variables to support a trait that gets their HTML format
        return Err(ActionError::VarInvalid(var_id.clone()));
      }

      self.html_config
        .format_input_template(html_template, &name_escaped)
        .and_then(|input_html| html.write_str(&input_html[..]))
        .map_err(|_e| ActionError::Other)?;
    }

    let stringval = StringValue::try_new(html).map_err(|_e| ActionError::Other)?;
    Ok(ActionResult::StartWith(stringval.boxed()))
  }
}



#[cfg(test)]
mod tests {
  use std::collections::HashSet;
  use super::{HtmlEscapedString, HtmlFormConfig, HtmlFormAction};
  use stepflow_base::{ObjectStore, ObjectStoreFiltered};
  use stepflow_data::{EmailVar, StateData, StateDataFiltered, Var, VarId, StringValue, StringVar, UriVar};
  use stepflow_step::{Step, StepId};
  use stepflow_test_util::test_id;
  use super::super::{ActionResult, Action, ActionId};

  #[test]
  fn html_format_input() {
    let mut html_config: HtmlFormConfig = Default::default();
    html_config.stringvar_html_template = "s({{name}},{{name}})".to_owned();
    html_config.urivar_html_template = "u({{name}},{{name}})".to_owned();
    html_config.emailvar_html_template = "e({{name}},{{name}})".to_owned();

    // simple case
    let escaped_n = HtmlEscapedString::new("n");
    let formatted = html_config.format_input_template(&html_config.stringvar_html_template, &escaped_n).unwrap();
    assert_eq!(formatted, "s(n,n)");

    // add prefix
    html_config.prefix_html_template = Some("p({{name}})".to_owned());
    let formatted_prefix = html_config.format_input_template(&html_config.stringvar_html_template, &escaped_n).unwrap();
    assert_eq!(formatted_prefix, "p(n)s(n,n)");

    // add wrap
    html_config.wrap_tag = Some("div".to_owned());
    let wrapped_prefix = html_config.format_input_template(&html_config.stringvar_html_template, &escaped_n).unwrap();
    assert_eq!(wrapped_prefix, "<div>p(n)s(n,n)</div>");

    // empty wrap
    html_config.wrap_tag = Some(String::new());
    let wrapped_empty = html_config.format_input_template(&html_config.stringvar_html_template, &escaped_n).unwrap();
    assert_eq!(wrapped_empty, "p(n)s(n,n)");
  }

  #[test]
  fn simple_form() {
    let var1 = StringVar::new(test_id!(VarId));
    let var2 = EmailVar::new(test_id!(VarId));
    let var3 = UriVar::new(test_id!(VarId));
    let var_ids = vec![var1.id().clone(), var2.id().clone(), var3.id().clone()];
    let step = Step::new(StepId::new(4), None, var_ids.clone());

    let state_data = StateData::new();
    let var_filter = var_ids.iter().map(|id| id.clone()).collect::<HashSet<_>>();
    let step_data_filtered = StateDataFiltered::new(&state_data, var_filter.clone());

    let mut var_store: ObjectStore<Box<dyn Var + Send + Sync>, VarId> = ObjectStore::new();
    var_store.register(Some("var 1".to_owned()), var1.boxed()).unwrap();
    var_store.register(Some("var 2".to_owned()), var2.boxed()).unwrap();
    var_store.register(Some("var 3".to_owned()), var3.boxed()).unwrap();

    let var_store_filtered = ObjectStoreFiltered::new(&var_store, var_filter);

    let mut exec = HtmlFormAction::new(test_id!(ActionId), Default::default());
    let action_result = exec.start(&step, None, &step_data_filtered, &var_store_filtered).unwrap();
    if let ActionResult::StartWith(html) = action_result {
      let html = html.downcast::<StringValue>().unwrap().val();
      assert_eq!(html, "<input name='var&#x20;1' /><input name='var&#x20;2' type='email' /><input name='var&#x20;3' type='url' />");
    } else {
      panic!("Did not get startwith value");
    }

    // customize the tags
    let mut html_config: HtmlFormConfig = Default::default();
    html_config.prefix_html_template = Some("p({{name}})".to_owned());
    html_config.stringvar_html_template = "l({{name}})s({{name}})".to_owned();
    html_config.urivar_html_template = "l({{name}})u({{name}})".to_owned();
    html_config.emailvar_html_template = "l({{name}})e({{name}})".to_owned();
    let mut custom_exec = HtmlFormAction::new(test_id!(ActionId), html_config);
    let custom_result = custom_exec.start(&step, None, &step_data_filtered, &var_store_filtered).unwrap();
    if let ActionResult::StartWith(html) = custom_result {
      let html = html.downcast::<StringValue>().unwrap().val();
      assert_eq!(html, "p(var&#x20;1)l(var&#x20;1)s(var&#x20;1)p(var&#x20;2)l(var&#x20;2)e(var&#x20;2)p(var&#x20;3)l(var&#x20;3)u(var&#x20;3)");
    } else {
      panic!("Did not get startwith value");
    }
  }

}
