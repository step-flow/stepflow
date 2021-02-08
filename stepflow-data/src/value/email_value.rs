use std::str::FromStr;
use once_cell::sync::Lazy;
use regex::Regex;
use super::{Value, BaseValue, InvalidValue};

define_value!(EmailValue, String, validate);

impl EmailValue {
  pub fn validate(val: &String) -> Result<(), InvalidValue> {
    if val.is_empty() {
      return Err(InvalidValue::Empty);
    }

    if extract_login(val).is_none() {
      return Err(InvalidValue::BadFormat)
    }

    Ok(())
  }
}

impl FromStr for EmailValue {
    type Err = InvalidValue;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
      EmailValue::try_new(s.to_owned())
    }
}

// based on https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html
fn extract_login(input: &str) -> Option<&str> {
  static REGEX_EMAIL: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?x)^(?P<login>[^@\s]+)@([[:word:]]+\.)*[[:word:]]+$").unwrap());
  REGEX_EMAIL.captures(input).and_then(|cap| {
    cap.name("login").map(|login| login.as_str())
  })
}

#[cfg(test)]
mod tests {
  use super::super::InvalidValue;

    use super::{ extract_login, EmailValue };

  #[test]
  fn test_extract_login() {
    // based on https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html
    assert_eq!(extract_login(r"I❤email@example.com"), Some(r"I❤email"));
    assert_eq!(extract_login(r"sdf+sdsfsd.as.sdsd@jhkk.d.rl"), Some(r"sdf+sdsfsd.as.sdsd"));
    assert_eq!(extract_login(r"More@Than@One@at.com"), None);
    assert_eq!(extract_login(r"Not an email@email"), None);
  }

  #[test]
  fn test_good_email() {
    let email = EmailValue::try_new("a@b.com".to_owned()).unwrap();
    assert_eq!(email.val(), "a@b.com");
  }

  #[test]
  fn test_bad_email() {
    let email_result = EmailValue::try_new("".to_owned());
    assert_eq!(email_result, Err(InvalidValue::Empty));

    let email_result = EmailValue::try_new("ab.com".to_owned());
    assert_eq!(email_result, Err(InvalidValue::BadFormat));
  }

  #[test]
  fn test_fromstr() {
    assert!(matches!("".parse::<EmailValue>(), Err(_))); 
    assert!(matches!("notemail".parse::<EmailValue>(), Err(_))); 
    assert_eq!("valid@email.com".parse::<EmailValue>().unwrap(), EmailValue::try_new("valid@email.com".to_owned()).unwrap());
  }
}