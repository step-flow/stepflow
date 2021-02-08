use super::{Value, InvalidValue, Var, VarId};

#[derive(Debug, Clone, serde::Serialize)]
pub struct ValidVal {
  val: Box<dyn Value>,
  validated_by: VarId,
}

impl ValidVal {
  pub fn try_new(val: Box<dyn Value>, validate_with: &Box<dyn Var + Send + Sync>) -> Result<Self, InvalidValue> {
    match validate_with.validate_val_type(&val) {
      Ok(_) => Ok(Self { 
        val: val, 
        validated_by: validate_with.id().clone() 
      }),
      Err(e) => Err(e),
    }
  }

  pub fn get_val(&self) -> &Box<dyn Value> {
    &self.val
  }
}

impl PartialEq for ValidVal {
    fn eq(&self, other: &Self) -> bool {
        self.val.eq_box(&other.val) && self.validated_by == other.validated_by
    }
}

#[cfg(test)]
mod tests {
  use stepflow_test_util::test_id;
  use super::super::{Var, VarId, EmailVar, EmailValue, StringVar, StringValue};
  use super::ValidVal;

  #[test]
  fn partial_eq() {
    const EMAIL1: &str = "a@b.com";
    const EMAIL2: &str = "ab@bc.com";

    // same var, same base value
    let email_var: Box<dyn Var + Send + Sync + 'static> = Box::new(EmailVar::new(test_id!(VarId)));
    let email_val1 = EmailValue::try_new(EMAIL1.to_owned()).unwrap();
    let valid_email = ValidVal::try_new(Box::new(email_val1.clone()), &email_var).unwrap();
    let valid_email_same = ValidVal::try_new(Box::new(email_val1), &email_var).unwrap();

    // same var, different base value
    let email_val2 = EmailValue::try_new(EMAIL2.to_owned()).unwrap();
    let valid_email_different = ValidVal::try_new(Box::new(email_val2), &email_var).unwrap();

    // different var, same base value
    let string_var: Box<dyn Var + Send + Sync + 'static> = Box::new(StringVar::new(test_id!(VarId)));
    let string_val = StringValue::try_new(EMAIL1.to_owned()).unwrap();
    let valid_string = ValidVal::try_new(Box::new(string_val), &string_var).unwrap();

    assert_eq!(valid_email, valid_email_same);
    assert_ne!(valid_email, valid_email_different);
    assert_ne!(valid_email, valid_string);
  }
}