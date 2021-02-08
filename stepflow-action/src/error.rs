use stepflow_data::VarId;


#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub enum ActionError {
  VarInvalid(VarId),
  Other,
}
