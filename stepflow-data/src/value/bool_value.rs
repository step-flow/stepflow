use super::{Value, BaseValue, InvalidValue};

define_value!(BoolValue, bool);


impl std::str::FromStr for BoolValue {
  type Err = InvalidValue;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match &s.to_lowercase()[..] {
      "true" => Ok(BoolValue::new(true)),
      "false" => Ok(BoolValue::new(false)),
      _ => Err(InvalidValue::WrongValue),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::{BoolValue, InvalidValue};

  #[test]
  fn from_str() {
    let true_val = "tRuE".parse::<BoolValue>().unwrap();
    assert_eq!(*true_val.val(), true);

    let false_val = "FaLse".parse::<BoolValue>().unwrap();
    assert_eq!(*false_val.val(), false);

    let bad_val_result = "hiya".parse::<BoolValue>();
    assert_eq!(bad_val_result, Err(InvalidValue::WrongValue));
  }
}