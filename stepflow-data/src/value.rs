use std::fmt::Debug;
use super::{BaseValue, InvalidValue};


pub trait Value: Debug + Sync + Send + stepflow_base::as_any::AsAny {
  fn get_baseval(&self) -> BaseValue;
  fn clone_box(&self) -> Box<dyn Value>;
  fn eq_box(&self, other: &Box<dyn Value>) -> bool;
}

// implement downcast helpers that have trait bounds to make it a little safer
impl dyn Value {
  pub fn downcast<T>(&self) -> Option<&T>
    where T: Value + std::any::Any
  {
    self.as_any().downcast_ref::<T>()
  }
  pub fn is<T>(&self) -> bool 
    where T: Value + std::any::Any
  {
    self.as_any().is::<T>()
  }
}

impl Clone for Box<dyn Value> {
    fn clone(&self) -> Box<dyn Value> {
        self.clone_box()
    }
}

impl PartialEq for Box<dyn Value> {
    fn eq(&self, other: &Box<dyn Value>) -> bool {
      self.eq_box(other)
    }
}

impl serde::Serialize for Box<dyn Value> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer
    {
      match self.get_baseval() {
          BaseValue::String(s) => s.serialize(serializer),
          BaseValue::Boolean(b) => b.serialize(serializer),
          BaseValue::Float(float) => float.serialize(serializer),
      }
    }
}

#[macro_use]
macro_rules! define_base_value {
  ($name:ident, $basetype:ident) => {
    #[derive(Debug, PartialEq, Clone)]
    pub struct $name {
      val: $basetype,
    }

    impl $name {
      pub fn val(&self) -> &$basetype {
        &self.val
      }
      pub fn boxed(self) -> Box<dyn Value> {
        Box::new(self)
      }
    }

    impl Value for $name {
      fn get_baseval(&self) -> BaseValue {
        self.val.clone().into()
      }
      fn clone_box(&self) -> Box<dyn Value> {
        Box::new(self.clone())
      }
      fn eq_box(&self, other: &Box<dyn Value>) -> bool {
        // check type is same
        if !other.is::<Self>() {
          return false;
        }

        // check baseval is same
        self.get_baseval() == other.get_baseval()
      }
    }
  };
}

#[macro_use]
macro_rules! define_value {
  ($name:ident, $basetype:ident) => {
    define_base_value!($name, $basetype);
    impl $name {
      pub fn new(val: $basetype) -> Self {
        $name { val }
      }
    }
  };

  ($name:ident, $basetype:ident, $validate_fn:ident) => {
    define_base_value!($name, $basetype);
    impl $name {
      pub fn try_new(val: String) -> Result<Self, InvalidValue> {
        Self::$validate_fn(&val)?;
        Ok(Self { val })
      }
    }
  };
}

mod uri_value;
pub use uri_value::UriValue;

mod string_value;
pub use string_value::StringValue;

mod email_value;
pub use email_value::EmailValue;

mod bool_value;
pub use bool_value::BoolValue;

mod true_value;
pub use true_value::TrueValue;


#[cfg(test)]
mod tests {
  use super::{EmailValue, Value, StringValue, TrueValue};

  #[test]
  fn val_downcast() {
    // try with reference
    let strval = StringValue::try_new("hi".to_owned()).unwrap();
    let r: &(dyn Value + 'static) = &strval;
    assert!(r.as_any().is::<StringValue>());

    // try with box ... if it fails, we're getting AsAny of the Box<T> as opposed to T
    let val: Box<dyn Value> = Box::new(strval.clone());
    assert!(val.as_any().is::<StringValue>());
    assert!(val.as_ref().as_any().is::<StringValue>());
    let stringval: Option<&StringValue> = val.downcast::<StringValue>();
    assert!(matches!(stringval, Some(_)));

    // try our helper fn
    assert_eq!(val.downcast::<StringValue>().unwrap().val(), "hi");
    assert_eq!(val.is::<StringValue>(), true);
    assert_eq!(val.downcast::<EmailValue>(), None);
    assert_eq!(val.is::<EmailValue>(), false);
  }

  #[test]
  fn partial_eq() {
    const EMAIL: &str = "a@b.com";
    let true_val: Box<dyn Value> = TrueValue::new().boxed();
    let email_val: Box<dyn Value> = EmailValue::try_new(EMAIL.to_owned()).unwrap().boxed();
    let string_val: Box<dyn Value> = StringValue::try_new(EMAIL.to_owned()).unwrap().boxed();
    assert!(email_val.clone() == email_val.clone());  // same thing
    assert!(true_val != email_val.clone());           // different types
    assert!(email_val.clone() != string_val);         // different types, same base value
  }
}
