use super::{Value, BaseValue, InvalidValue};

define_value!(StringValue, String, validate);

impl StringValue {
  pub fn validate(val: &String) -> Result<(), InvalidValue> {
    if val.is_empty() {
      return Err(InvalidValue::Empty);
    }
    Ok(())
  }
}

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
    let string_value = StringValue::try_new("hi".to_owned()).unwrap();
    assert_eq!(string_value.val(), "hi");
  }

  #[test]
  fn test_bad_string() {
    assert_eq!(StringValue::try_new("".to_owned()), Err(InvalidValue::Empty));
  }

  #[test]
  fn test_fromstr() {
    assert!(matches!("".parse::<StringValue>(), Err(_))); 
    assert_eq!("valid".parse::<StringValue>().unwrap(), StringValue::try_new("valid".to_owned()).unwrap());
  }
}