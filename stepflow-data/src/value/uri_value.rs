use http::Uri;
use super::{Value, BaseValue, InvalidValue};

define_value!(UriValue, String, validate);

impl UriValue {
  pub fn validate(val: &String) -> Result<(), InvalidValue> {
    let _uri: Uri = val.parse().map_err(|_e| InvalidValue::BadFormat)?;
    Ok(())
  }

  pub fn uri_val(&self) -> Uri {
    self.val.parse::<Uri>().unwrap()
  }
}

impl std::str::FromStr for UriValue {
  type Err = InvalidValue;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    UriValue::try_new(s.to_owned())
  }
}
