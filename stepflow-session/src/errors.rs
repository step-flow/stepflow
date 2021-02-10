use stepflow_base::IdError;
use stepflow_data::var::VarId;
use stepflow_step::StepId;
use stepflow_action::{ActionError, ActionId};
use crate::SessionId;

#[derive(Debug, PartialEq, serde::Serialize, Clone)]
pub enum Error {
  // ID errors
  VarId(IdError<VarId>),
  StepId(IdError<StepId>),
  ActionId(IdError<ActionId>),
  SessionId(IdError<SessionId>),

  // data errors
  InvalidValue(stepflow_data::InvalidValue),
  InvalidVars(stepflow_data::InvalidVars),
  InvalidStateDataError,

  // action + step execution errors
  NoStateToEval,
  ActionError(stepflow_action::ActionError),

  // something we try to not use
  Other,
}

impl From<stepflow_data::InvalidValue> for Error {
  fn from(err: stepflow_data::InvalidValue) -> Self {
    Error::InvalidValue(err)
  }
}

impl From<ActionError> for Error {
    fn from(err: ActionError) -> Self {
      Error::ActionError(err)
    }
}

macro_rules! from_id_error {
  ($id_type:ident) => {
    impl From<IdError<$id_type>> for Error {
      fn from(err: IdError<$id_type>) -> Self {
        Error::$id_type(err)
      }
    }
  };
}

from_id_error!(VarId);
from_id_error!(StepId);
from_id_error!(ActionId);
from_id_error!(SessionId);