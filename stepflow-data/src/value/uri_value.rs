use std::borrow::{Borrow, Cow};
use super::{Value, BaseValue, InvalidValue};
use http::Uri;

#[derive(Debug, PartialEq, Clone)]
pub struct UriValue {
  val: Cow<'static, str>,
}

impl UriValue {
  pub fn try_new<STR>(val: STR) -> Result<Self, InvalidValue> 
      where STR: Into<Cow<'static, str>>
  {
    let val = val.into();
    Self::validate(&val)?;
    Ok(Self { val })
  }

  pub fn validate(val: &Cow<'static, str>) -> Result<(), InvalidValue> {
    let _uri: Uri = val.parse().map_err(|_e| InvalidValue::BadFormat)?;
    Ok(())
  }

  pub fn val(&self) -> &str {
    self.val.borrow()
  }

  pub fn uri_val(&self) -> Uri {
    self.val.parse::<Uri>().unwrap()
  }

  pub fn boxed(self) -> Box<dyn Value> {
    Box::new(self)
  }
}

define_value_impl!(UriValue);

impl std::str::FromStr for UriValue {
  type Err = InvalidValue;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    UriValue::try_new(s.to_owned())
  }
}


#[cfg(test)]
mod tests {
  use super::{InvalidValue, UriValue};


  const GOOD_URI:&str = "/hi";
  const BAD_URI: &str = "$$$å";

  #[test]
  fn test_good_uri() {
    let uri_value = UriValue::try_new(GOOD_URI).unwrap();
    assert_eq!(uri_value.val(), GOOD_URI);
  }

  #[test]
  fn test_bad_uri() {
    assert_eq!(UriValue::try_new(BAD_URI), Err(InvalidValue::BadFormat));
  }

  #[test]
  fn test_fromstr() {
    assert!(matches!(BAD_URI.parse::<UriValue>(), Err(_)));
    assert_eq!(GOOD_URI.parse::<UriValue>().unwrap(), UriValue::try_new(GOOD_URI).unwrap());
  }
}