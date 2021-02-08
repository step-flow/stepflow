#[derive(Debug, PartialEq, serde::Serialize, Clone)]
pub enum IdError<TID> {
  CannotParse(String),
  IdNotReserved(TID),
  IdAlreadyExists(TID),
  IdMissing(TID),
  IdUnexpected(TID),
  NameAlreadyExists(String),
  NoSuchName(String),
}