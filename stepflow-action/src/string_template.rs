use std::collections::HashMap;


// NOTE: This hack is pretty unreliable and can probably avoid the string re-allocations
// In the future, if we don't replace every var, we should return an UnusedParam error
pub fn render_template<ES>(escaped_template: &ES, params: HashMap<&'static str, ES>) -> String
    where ES: AsRef<str>
{
  let mut escaped_template: &str = escaped_template.as_ref();
  let mut result = String::new();

  for (k, v) in params {
    let mut full_key = String::with_capacity(k.len() + 4 /* {{}} */);
    full_key.push_str("{{");
    full_key.push_str(&k[..]);
    full_key.push_str("}}");

    result = escaped_template.replace(&full_key[..], v.as_ref());
    escaped_template = &result[..];
  }
  result
}

pub trait EscapedString : AsRef<str> + std::fmt::Debug + Send + Sync + 'static {
  fn from_unescaped(unescaped_str: &str) -> Self;
  fn already_escaped(escaped_str: String) -> Self;
}

#[derive(Debug)]
pub struct HtmlEscapedString(String);
impl EscapedString for HtmlEscapedString {
  fn from_unescaped(unescaped_str: &str) -> Self {
    HtmlEscapedString(htmlescape::encode_attribute(unescaped_str))
  }
  fn already_escaped(escaped_str: String) -> Self {
    HtmlEscapedString(escaped_str)
  }
}

impl HtmlEscapedString {
  pub fn len(&self) -> usize {
    self.0.len()
  }
}

impl AsRef<str> for HtmlEscapedString {
    fn as_ref(&self) -> &str {
        &(self.0)[..]
    }
}

#[derive(Debug)]
pub struct UriEscapedString(String);
impl EscapedString for UriEscapedString {
  fn from_unescaped(unescaped_str: &str) -> Self {
    UriEscapedString(urlencoding::encode(unescaped_str))
  }
  fn already_escaped(escaped_str: String) -> Self {
    UriEscapedString(escaped_str)
  }
}
impl AsRef<str> for UriEscapedString {
    fn as_ref(&self) -> &str {
        &(self.0)[..]
    }
}


#[cfg(test)]
mod tests {
  use std::collections::HashMap;
  use super::render_template;

  struct Escaped(&'static str);
  impl AsRef<str> for Escaped {
      fn as_ref(&self) -> &str {
        self.0
    }
  }

  #[test]
  fn handlebars_basic() {
    let mut params = HashMap::new();
    params.insert("name", Escaped("bob"));
    params.insert("value", Escaped("myvalue"));
    let output = render_template::<Escaped>(&Escaped("name{{name}}, value{{value}}"), params);
    assert_eq!(output, "namebob, valuemyvalue");
  }
}