//! [`Var`]s are placeholders for [`Value`]s. They can be used to define what values are needed
//! later without creating the value.
//!
//! When needed, they can be downcast to their original type via `Var::downcast` and `Var::is`.
use stepflow_base::{ObjectStoreContent, IdError, generate_id_type};
use super::InvalidValue;
use super::value::Value;

generate_id_type!(VarId);

pub trait Var: std::fmt::Debug + stepflow_base::as_any::AsAny {
  fn id(&self) -> &VarId;
  fn value_from_str(&self, s: &str) -> Result<Box<dyn Value>, InvalidValue>;
  fn validate_val_type(&self, val: &Box<dyn Value>) -> Result<(), InvalidValue>;
}

// implement downcast helpers that have trait bounds to make it a little safer
impl dyn Var + Send + Sync {
  pub fn downcast<T>(&self) -> Option<&T>
    where T: Var + std::any::Any
  {
    self.as_any().downcast_ref::<T>()
  }
  pub fn is<T>(&self) -> bool 
    where T: Var + std::any::Any
  {
    self.as_any().is::<T>()
  }
}

impl ObjectStoreContent for Box<dyn Var + Sync + Send> {
  type IdType = VarId;

  fn new_id(id_val: u32) -> Self::IdType {
    VarId::new(id_val)
  }

  fn id(&self) -> &Self::IdType {
    self.as_ref().id()
  }
}

macro_rules! define_var {
  ($name:ident, $valuetype:ident) => {

    #[derive(Debug)]
    pub struct $name {
      id: VarId,
    }
    impl $name {
      /// Create a new var
      pub fn new(id: VarId) -> Self {
        Self { id }
      }

      /// Box the value
      pub fn boxed(self) -> Box<dyn Var + Send + Sync> {
        Box::new(self)
      }
    }
    impl Var for $name {
      /// Gets the ID
      fn id(&self) -> &VarId { &self.id }

      /// Convert a &str to this Var's corresponding value
      fn value_from_str(&self, s: &str) -> Result<Box<dyn Value>, InvalidValue> {
        Ok(Box::new(s.parse::<$valuetype>()?) as Box<dyn Value>)
      }

      /// Validate the value type corresponds to this Var
      fn validate_val_type(&self, val: &Box<dyn Value>) -> Result<(), InvalidValue> {
        if val.is::<$valuetype>() {
          Ok(())
        } else {
          Err(InvalidValue::WrongType)
        }
      }
    }
  };
}

use super::value::EmailValue;
define_var!(EmailVar, EmailValue);

use super::value::StringValue;
define_var!(StringVar, StringValue);

use super::value::TrueValue;
define_var!(TrueVar, TrueValue);

use super::value::UriValue;
define_var!(UriVar, UriValue);

use super::value::BoolValue;
define_var!(BoolVar, BoolValue);


#[cfg(test)]
pub fn test_var_val() -> (Box<dyn Var + Send + Sync>, Box<dyn Value>) {
  let var = Box::new(StringVar::new(stepflow_test_util::test_id!(VarId)));
  let val: Box<dyn Value> = StringValue::try_new("test").unwrap().boxed();
  (var, val)
}

#[cfg(test)]
mod tests {
  use stepflow_test_util::test_id;
  use crate::value::{Value, StringValue, EmailValue};
  use super::{Var, VarId, EmailVar, StringVar, UriVar, InvalidValue};

  #[test]
  fn validate_val_type() {
    let email_addr = "is@email.com";
    let email_var = EmailVar::new(test_id!(VarId));

    let email_strval: Box<dyn Value> = StringValue::try_new(email_addr).unwrap().boxed();
    let email_emailval: Box<dyn Value> = EmailValue::try_new(email_addr).unwrap().boxed();
    assert!(matches!(email_var.validate_val_type(&email_strval), Err(InvalidValue::WrongType)));
    assert!(matches!(email_var.validate_val_type(&email_emailval), Ok(())));
  }

  #[test]
  fn downcast() {
    let stringvar = StringVar::new(test_id!(VarId));
    let stringvar_boxed = stringvar.boxed();
    assert!(matches!(stringvar_boxed.as_any().downcast_ref::<StringVar>(), Some(_)));
    assert!(matches!(stringvar_boxed.as_any().downcast_ref::<UriVar>(), None));

    // try our helper 
    assert!(matches!(stringvar_boxed.downcast::<StringVar>(), Some(_)));
    assert_eq!(stringvar_boxed.is::<StringVar>(), true);
    assert!(matches!(stringvar_boxed.downcast::<UriVar>(), None));
    assert_eq!(stringvar_boxed.is::<UriVar>(), false);
  }
}