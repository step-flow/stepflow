use std::collections::HashMap;
use super::var::VarId;

#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "serde-support", derive(serde::Serialize))]
pub enum InvalidValue {
  WrongType,
  BadFormat,
  Empty,
  WrongValue,
}

impl std::error::Error for InvalidValue {}

impl std::fmt::Display for InvalidValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "{:?}", self)
    }
}


#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde-support", derive(serde::Serialize))]
pub struct InvalidVars(pub HashMap<VarId, InvalidValue>);
impl InvalidVars {
  pub fn new(invalid: HashMap<VarId, InvalidValue>) -> Self {
    Self(invalid)
  }
}
