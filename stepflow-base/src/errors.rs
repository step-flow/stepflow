#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature="serde-support", derive(serde::Serialize))]
pub enum IdError<TID> {
  CannotParse(String),
  IdNotReserved(TID),
  IdAlreadyExists(TID),
  IdMissing(TID),
  IdUnexpected(TID),
  IdHasNoName(TID),
  NameAlreadyExists(String),
  NoSuchName(String),
}