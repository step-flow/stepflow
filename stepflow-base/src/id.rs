/// Macro to create an ID to be used by an [`ObjectStore`](crate::ObjectStore)
#[macro_export]
macro_rules! generate_id_type {
  ($struct_name:ident) => {
    #[derive(Hash, Clone, Copy, Debug, serde::Serialize, PartialEq, Eq)]
    pub struct $struct_name(u32);
    impl $struct_name {
      pub fn new(val: u32) -> Self {
        $struct_name(val)
      }
      pub fn val(&self) -> u32 {
        self.0
      }
    }
    impl std::fmt::Display for $struct_name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
      }
    }
    impl std::str::FromStr for $struct_name {
      type Err = IdError<$struct_name>;

      fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val = s.parse::<u32>().map_err(|_e| IdError::CannotParse(s.to_owned()))?;
        Ok(Self::new(val))
      }
    }

    impl std::default::Default for $struct_name {
      fn default() -> Self {
        Self::new(0)
      }
    }
  };
}

#[cfg(test)]
mod tests {
  use crate::IdError;
  use stepflow_test_util::test_id;

  generate_id_type!(TestId);

  #[test]
  fn new_id() {
    let test_id = TestId::new(10);

    // check eq & val()
    assert_eq!(test_id.val(), 10);
    assert_ne!(test_id.val(), TestId::new(15).val());
  }

  #[test]
  fn test_testid() {
    let test_id1: TestId =  test_id!(TestId);
    let test_id2: TestId = test_id!(TestId);
    assert_ne!(test_id1.val(), test_id2.val());
  }

  #[test]
  fn from_str() {
    let test_id = "48".parse::<TestId>().unwrap();
    assert_eq!(test_id, TestId::new(48));
  }
}

