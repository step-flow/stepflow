use stepflow::object::{ObjectStore, IdError};
use stepflow::data::{Var, VarId, StringVar, EmailVar, TrueVar};
use stepflow::step::{Step, StepId};
use stepflow::{Session, Error};
use stepflow_action::{ActionId, EscapedString, StringTemplateAction, SetDataAction, UriEscapedString};
use stepflow_data::StateData;

pub enum VarType { String, Email, True }

pub struct VarInfo(pub &'static str, pub VarType);

// register Vars and return the IDs
pub fn register_vars(session: &mut Session, varinfos: &Vec<VarInfo>) -> Result<Vec<VarId>, Error> {
  let var_store = session.var_store_mut();
  let vars = varinfos
    .iter()
    .map(|varinfo| {
      let cb: fn(VarId) -> Result<Box<dyn Var + Send + Sync>, _> = match varinfo.1 {
        VarType::String => |id: VarId| Ok(StringVar::new(id).boxed()),
        VarType::Email => |id: VarId| Ok(EmailVar::new(id).boxed()),
        VarType::True => |id: VarId| Ok(TrueVar::new(id).boxed()),
      };
      var_store.insert_new_named(varinfo.0, cb)
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
            Some(inputs) => Some(names_to_var_ids(session.var_store(), inputs)?),
            None => None,
        };
        let output_vars = names_to_var_ids(session.var_store(),  stepinfo.2)?;
        session.step_store_mut().insert_new_named(
            stepinfo.0,
            |id| Ok(Step::new(id, input_vars, output_vars)))
            .map_err(|id_error| Error::from(id_error))
      })
      .collect::<Result<Vec<StepId>, Error>>()?;
  Ok(step_ids)
}


// in the future make this a generic for any objectstore
fn names_to_var_ids(var_store: &ObjectStore<Box<dyn Var + Send + Sync>, VarId>, var_names: Vec<&str>)
        -> Result<Vec<VarId>, Error> 
{
    var_names.into_iter()
        .map(|name| {
            var_store.id_from_name(name)
                .map(|id_ref| id_ref.clone())
                .ok_or_else(|| Error::VarId(IdError::NoSuchName(name.to_owned())))
        })
        .collect::<Result<Vec<VarId>, Error>>()
}

pub enum ActionInfo {
  UriAction { step_name: Option<&'static str>, base_path: String },
  SetDataAction { step_name: Option<&'static str>, statedata: StateData, after_attempt: u64},
}

pub fn register_actions(session: &mut Session, actioninfos: Vec<ActionInfo>) -> Result<Vec<ActionId>, Error> {
  actioninfos
    .into_iter()
    .map(|info| {
      let action_id = session.action_store_mut().reserve_id();
      let step_name_action;
      let action = match info {
        ActionInfo::UriAction { step_name, base_path } => {
          step_name_action = step_name;
          StringTemplateAction::new(action_id, UriEscapedString::already_escaped(format!("{}/{{{{step}}}}", base_path))).boxed()
        }
        ActionInfo::SetDataAction { step_name, statedata, after_attempt } => {
          step_name_action = step_name;
          SetDataAction::new(action_id, statedata, after_attempt).boxed()
        }
      };

      let step_id = step_name_action.map(|step_name| session.step_store().id_from_name(step_name).unwrap().clone());
      session.action_store_mut().register(action).unwrap();
      session.set_action_for_step(action_id, step_id.as_ref())?;
      return Ok(action_id);
    })
    .collect::<Result<_,_>>()
}
