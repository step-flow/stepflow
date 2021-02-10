use crate::InvalidValue;

use super::{Value, BaseValue};

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub struct TrueValue;

impl TrueValue {
  pub fn new() -> Self { Self {} }
  pub fn val(&self) -> bool { true }
  pub fn boxed(self) -> Box<dyn Value> { Box::new(self) }
}

impl Value for TrueValue {
  fn get_baseval(&self) -> BaseValue {
    BaseValue::Boolean(true)
  }

  fn clone_box(&self) -> Box<dyn Value> {
    Box::new(self.clone())
  }

  fn eq_box(&self, other: &Box<dyn Value>) -> bool {
    // no value -- just an existence check so if the other is the same type, they're equal
    other.is::<Self>()
  }
}

impl std::str::FromStr for TrueValue {
  type Err = InvalidValue;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.to_lowercase() == "true" {
      Ok(TrueValue::new())
    } else  {
      Err(InvalidValue::WrongValue)
    }
  }
}


#[cfg(test)]
mod tests {
  use crate::{BaseValue, InvalidValue, value::StringValue};
  use super::{TrueValue, Value};

  #[test]
  fn is_true() {
    let true_val = TrueValue::new();
    assert_eq!(true_val.val(), true);
    assert!(matches!(true_val.get_baseval(), BaseValue::Boolean(f) if f == true));
  }

  #[test]
  fn partial_eq() {
    let true_val1 = TrueValue::new();
    let true_val2 = TrueValue::new();
    assert_eq!(true_val1, true_val2);

    let b1: Box<dyn Value> = Box::new(true_val1);
    let b2: Box<dyn Value> = Box::new(true_val2);
    assert!(b1 == b2);

    let string_val = StringValue::try_new("true".to_owned()).unwrap();
    let s: Box<dyn Value> = Box::new(string_val);
    assert!(b1 != s);
  }

  #[test]
  fn from_str() {
    assert_eq!("true".parse::<TrueValue>(), Ok(TrueValue::new()));
    assert_eq!("tRuE".parse::<TrueValue>(), Ok(TrueValue::new()));
    assert_eq!("tRuEe".parse::<TrueValue>(), Err(InvalidValue::WrongValue));
  }
}