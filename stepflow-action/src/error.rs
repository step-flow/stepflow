use stepflow_data::var::VarId;


#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub enum ActionError {
  VarInvalid(VarId),
  Other,
}
