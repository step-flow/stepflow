use std::borrow::Cow;

/// The base store for [`Value`](crate::value::Value). All values must support storing and retrieving data as one of these types.
#[derive(PartialEq)]
pub enum BaseValue {
  String(String),
  Boolean(bool),
  Float(f64),
}

impl From<String> for BaseValue {
    fn from(s: String) -> Self {
      BaseValue::String(s)
    }
}

impl From<Cow<'static, str>> for BaseValue {
  fn from(s: Cow<'static, str>) -> Self {
    BaseValue::String(s.into_owned())
  }
}

impl From<bool> for BaseValue {
    fn from(b: bool) -> Self {
      BaseValue::Boolean(b)
    }
}

impl From<f64> for BaseValue {
    fn from(float: f64) -> Self {
      BaseValue::Float(float)
    }
}
