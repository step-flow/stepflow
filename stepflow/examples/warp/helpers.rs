use stepflow::object::{ObjectStore, IdError};
use stepflow::data::{Var, VarId, StringVar, EmailVar, TrueVar};
use stepflow::step::{Step, StepId};
use stepflow::{Session, Error};
use stepflow_action::{Action, ActionId, SetDataAction, UrlStepAction};
use stepflow_data::StateData;

pub enum VarType { String, Email, True }

pub struct VarInfo(pub &'static str, pub VarType);

// register Vars and return the IDs
pub fn register_vars(session: &mut Session, varinfos: &Vec<VarInfo>) -> Result<Vec<VarId>, Error> {
  let varstore = session.varstore_mut();
  let vars = varinfos
    .iter()
    .map(|varinfo| {
      let cb: fn(VarId) -> Result<Box<dyn Var + Send + Sync>, _> = match varinfo.1 {
        VarType::String => |id: VarId| Ok(StringVar::new(id).boxed()),
        VarType::Email => |id: VarId| Ok(EmailVar::new(id).boxed()),
        VarType::True => |id: VarId| Ok(TrueVar::new(id).boxed()),
      };
      varstore.insert_new(Some(varinfo.0.to_owned()), cb)
    })
    .collect::<Result<Vec<VarId>, _>>()?;
  Ok(vars)
}

// name, inputs, outputs
pub struct StepInfo(pub &'static str, pub Option<Vec<&'static str>>, pub Vec<&'static str>);

// register steps with names and return the IDs
pub fn register_steps(session: &mut Session, stepinfos: Vec<StepInfo>) -> Result<Vec<StepId>, Error> {
  let step_ids = stepinfos
      .into_iter()
      .map(|stepinfo| {
        let input_vars = match stepinfo.1 {
            Some(inputs) => Some(names_to_var_ids(session.varstore(), inputs)?),
            None => None,
        };
        let output_vars = names_to_var_ids(session.varstore(),  stepinfo.2)?;
        session.step_store_mut().insert_new(
            Some(stepinfo.0.to_owned()), 
            |id| Ok(Step::new(id, input_vars, output_vars)))
            .map_err(|id_error| Error::from(id_error))
      })
      .collect::<Result<Vec<StepId>, Error>>()?;
  Ok(step_ids)
}


// in the future make this a generic for any objectstore
fn names_to_var_ids(varstore: &ObjectStore<Box<dyn Var + Send + Sync>, VarId>, var_names: Vec<&str>)
        -> Result<Vec<VarId>, Error> 
{
    var_names.into_iter()
        .map(|name| {
            varstore.id_from_name(name)
                .map(|id_ref| id_ref.clone())
                .ok_or_else(|| Error::VarId(IdError::NoSuchName(name.to_owned())))
        })
        .collect::<Result<Vec<VarId>, Error>>()
}

pub enum ActionInfo {
  UrlAction { step_name: Option<&'static str>, base_path: String },
  SetDataAction { step_name: Option<&'static str>, statedata: StateData, after_attempt: u64},
}

pub fn register_actions(session: &mut Session, actioninfos: Vec<ActionInfo>) -> Result<Vec<ActionId>, Error> {
  actioninfos
    .into_iter()
    .map(|info| {
      let action_id = session.action_store().reserve_id()?;
      let step_name_action;
      let _action = match info {
        ActionInfo::UrlAction { step_name, base_path } => {
          step_name_action = step_name;
          UrlStepAction::new(action_id, base_path.parse().unwrap()).boxed()
        }
        ActionInfo::SetDataAction { step_name, statedata, after_attempt } => {
          step_name_action = step_name;
          SetDataAction::new(action_id, statedata, after_attempt).boxed()
        }
      };

      let step_id = step_name_action.map(|step_name| session.step_store().id_from_name(step_name).unwrap().clone());
      session.set_action_for_step(action_id, step_id.as_ref())?;
      return Ok(action_id);
    })
    .collect::<Result<_,_>>()
}
