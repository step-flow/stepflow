use std::borrow::{Borrow, Cow};
use super::{Value, BaseValue, InvalidValue};

#[derive(Debug, PartialEq, Clone)]
pub struct StringValue {
  val: Cow<'static, str>,
}

impl StringValue {
  pub fn try_new<STR>(val: STR) -> Result<Self, InvalidValue> 
      where STR: Into<Cow<'static, str>>
  {
    let val = val.into();
    Self::validate(&val)?;
    Ok(Self { val })
  }

  pub fn validate(val: &Cow<'static, str>) -> Result<(), InvalidValue> {
    if val.is_empty() {
      return Err(InvalidValue::Empty);
    }
    Ok(())
  }

  pub fn val(&self) -> &str {
    self.val.borrow()
  }
  pub fn boxed(self) -> Box<dyn Value> {
    Box::new(self)
  }
}

define_value_impl!(StringValue);

impl std::str::FromStr for StringValue {
  type Err = InvalidValue;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    StringValue::try_new(s.to_owned())
  }
}


#[cfg(test)]
mod tests {
  use super::{InvalidValue, StringValue};

  #[test]
  fn test_good_string() {
    let string_value = StringValue::try_new("hi").unwrap();
    assert_eq!(string_value.val(), "hi");
  }

  #[test]
  fn test_bad_string() {
    assert_eq!(StringValue::try_new(""), Err(InvalidValue::Empty));
  }

  #[test]
  fn test_fromstr() {
    assert!(matches!("".parse::<StringValue>(), Err(_))); 
    assert_eq!("valid".parse::<StringValue>().unwrap(), StringValue::try_new("valid").unwrap());
  }
}