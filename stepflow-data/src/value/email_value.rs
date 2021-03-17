use std::borrow::{Borrow, Cow};
use std::str::FromStr;
use super::{Value, BaseValue, InvalidValue};


/// The implementation for an email [`value`](crate::value::Value).
///
/// NOTE: this is a really basic e-mail validity check and misses several cases.
#[derive(Debug, PartialEq, Clone)]
pub struct EmailValue {
  val: Cow<'static, str>,
}

impl EmailValue {
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

    if extract_login(val).is_none() {
      return Err(InvalidValue::BadFormat)
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

fn is_valid_email_local_part_char(c: char) -> bool {
  if c.is_alphanumeric() {
    return true;
  }
  match c {
    '!' | '#' | '$' | '%' | '&' | '*' | '+' | '-' | '/' | '=' | '?' | '^' | '_' | '`' | '{' | '|' | '}' | '~' => true,
    _ => false
  }
}
fn extract_login(input: &str) -> Option<&str> {
  #[derive(PartialEq, Debug)]
  enum ExtractState {
    LoginAnyLocalPartChar,       // login: next char must be valid in the "local-part" of an email
    LoginAnyLocalPartCharAndDot,
    Domain
  }

  let mut end_range = 0;
  let mut state = ExtractState::LoginAnyLocalPartChar;  // first char must be alphanum
  let mut login: &str = "";
  for c in input.chars() {
    // never valid
    if c.is_whitespace() {
      return None;
    }
    end_range += 1;

    state = match state {
      ExtractState::LoginAnyLocalPartChar |
      ExtractState::LoginAnyLocalPartCharAndDot => {
        if is_valid_email_local_part_char(c) {
          ExtractState::LoginAnyLocalPartCharAndDot
        } else if state == ExtractState::LoginAnyLocalPartCharAndDot && c == '.' {
          ExtractState::LoginAnyLocalPartChar
        } else if c == '@' {
          login = input.get(0..end_range-1)?;
          if login.chars().last()? == '.' {
            // look back one char to make sure we don't end in a dot
            return None;
          }
          ExtractState::Domain
        } else {
          return None;
        }
      }
      ExtractState::Domain => {
        match c {
          '@' => return None,
          _ => ExtractState::Domain,
        }
      }
    }
  }

  if login.is_empty() {
    // this should be impossible
    None
  } else {
    Some(login)
  }
}

define_value_impl!(EmailValue);

impl FromStr for EmailValue {
    type Err = InvalidValue;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
      EmailValue::try_new(s.to_owned())
    }
}


#[cfg(test)]
mod tests {
  use super::super::InvalidValue;
  use super::{ extract_login, EmailValue };

  #[test]
  fn test_extract_valid_email() {
    // from https://gist.github.com/cjaoude/fd9910626629b53c4d25
    // FUTURE: we don't handle unicode graphmemes to avoid growing our data segment with unicode tables. it should be an optional features
    let emails = vec![
      // valid
      ("email@example.com", "email"),
      ("firstname.lastname@example.com", "firstname.lastname"),
      ("email@subdomain.example.com", "email"),
      ("firstname+lastname@example.com", "firstname+lastname"),
      ("email@123.123.123.123", "email"),
      ("email@[123.123.123.123]", "email"),
      // ("“email”@example.com", "“email”"),
      ("1234567890@example.com", "1234567890"),
      ("email@example-one.com", "email"),
      ("_______@example.com", "_______"),
      ("email@example.name", "email"),
      ("email@example.museum", "email"),
      ("email@example.co.jp", "email"),
      ("firstname-lastname@example.com", "firstname-lastname"),
      // strange
      // ("much.”more\\ unusual”@example.com", "much.”more\\ unusual”"),
      // ("very.unusual.”@”.unusual.com@example.com", "very.unusual.”"),
      // ("very.”(),:;<>[]”.VERY.”very@\\\\ \"very”.unusual@strange.example.com", "very.”(),:;<>[]”.VERY.”very"),
    ];
    for (email, login) in emails {
      println!("Checking GOOD {}", email);
      let extracted_login = extract_login(email).unwrap();
      assert_eq!(extracted_login, login);
    }
  }

  #[test]
  fn test_extract_invalid_email() {
    // from https://gist.github.com/cjaoude/fd9910626629b53c4d25
    let bad_emails = vec![
      "plainaddress",
      "#@%^%#$@#$@#.com",
      "@example.com",
      "Joe Smith <email@example.com>",
      "email.example.com",
      "email@example@example.com",
      ".email@example.com",
      "email.@example.com",
      "email..email@example.com",
      "あいうえお@example.com",
      "email@example.com (Joe Smith)",
      // "email@example",
      // "email@-example.com",
      // "email@example.web",
      // "email@111.222.333.44444",
      // "email@example..com",
      "Abc..123@example.com",

      // strange
      "”(),:;<>[\\]@example.com",
      "just”not”right@example.com",
      "this\\ is\"really\"not\\allowed@example.com",
    ];
    for bad_email in bad_emails {
      println!("Checking BAD {}", bad_email);
      assert_eq!(extract_login(bad_email), None);
    }
  }

  #[test]
  fn test_good_email() {
    let email = EmailValue::try_new("a@b.com").unwrap();
    assert_eq!(email.val(), "a@b.com");
  }

  #[test]
  fn test_bad_email() {
    let email_result = EmailValue::try_new("");
    assert_eq!(email_result, Err(InvalidValue::Empty));

    let email_result = EmailValue::try_new("ab.com");
    assert_eq!(email_result, Err(InvalidValue::BadFormat));
  }

  #[test]
  fn test_fromstr() {
    assert!(matches!("".parse::<EmailValue>(), Err(_))); 
    assert!(matches!("notemail".parse::<EmailValue>(), Err(_))); 
    assert_eq!("valid@email.com".parse::<EmailValue>().unwrap(), EmailValue::try_new("valid@email.com").unwrap());
  }
}