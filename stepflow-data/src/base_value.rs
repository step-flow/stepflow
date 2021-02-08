
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
