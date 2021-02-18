use stepflow_base::IdError;
use stepflow_data::var::VarId;
use stepflow_step::StepId;

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub enum ActionError {
  // ID errors
  VarId(IdError<VarId>),
  StepId(IdError<StepId>),
  Other,
}
