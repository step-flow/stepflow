use std::collections::{HashMap, HashSet};
use stepflow_base::{ObjectStore, ObjectStoreContent, ObjectStoreFiltered, IdError, generate_id_type};
use stepflow_data::{StateData, StateDataFiltered, Var, VarId, Value};
use stepflow_step::{Step, StepId};
use stepflow_action::{ActionResult, ActionId};
use super::{Error, dfs};

mod action_object_store;
pub use action_object_store::ActionObjectStore;

generate_id_type!(SessionId);

#[derive(Debug)]
pub struct Session {
  id: SessionId,
  state_data: StateData,
  step_actions: HashMap<StepId, ActionId>,

  step_store: ObjectStore<Step, StepId>,
  action_store: ActionObjectStore,
  varstore: ObjectStore<Box<dyn Var + Send + Sync>, VarId>,

  step_id_all: StepId,
  step_id_root: StepId,

  step_id_dfs: dfs::DepthFirstSearch,
}

impl ObjectStoreContent for Session {
    type IdType = SessionId;

    fn new_id(id_val: u32) -> Self::IdType {
        SessionId::new(id_val)
    }

    fn id(&self) -> &Self::IdType {
        &self.id
    }
}

impl Session {
  // returns new session + first step id
  // we ask generate the root step to make sure the IDs don't conflict
  pub fn new(id: SessionId) -> Self {
    Self::with_capacity(id, 0, 0, 0)
  }

  pub fn with_capacity(id: SessionId, var_capacity: usize, step_capacity: usize, action_capacity: usize) -> Self {
    // create the step store
    let mut step_store = ObjectStore::with_capacity(step_capacity);

    // create a step ID for the action-all action
    let step_id_all = step_store.insert_new(
      Some("STEP_ID_ACTION_ALL".to_owned()),
      |id| Ok(Step::new(id, None, vec![]))).unwrap();

    // create the root step
    // this is so the first advance goes to the real root step (to use general dfs flow like checking canenter)
    let step_id_root = step_store.insert_new(
      Some("SESSION_ROOT".to_owned()),
      |id| Ok(Step::new(id, None, vec![]))).unwrap();
    
    Session {
      id,
      state_data: StateData::new(),
      step_actions: HashMap::new(),
      step_store,
      action_store: ActionObjectStore::with_capacity(action_capacity),
      varstore: ObjectStore::with_capacity(var_capacity),
      step_id_all: step_id_all,
      step_id_root: step_id_root,
      step_id_dfs: dfs::DepthFirstSearch::new(step_id_root),
    }
  }


  pub fn id(&self) -> &SessionId {
    &self.id
  }

  pub fn state_data(&self) -> &StateData {
    &self.state_data
  }

  pub fn current_step(&self) -> Result<&StepId, Error> {
    self.step_id_dfs.current().ok_or_else(|| Error::NoStateToEval)
  }

  pub fn step_store(&self) -> &ObjectStore<Step, StepId> {
    &self.step_store
  }

  pub fn step_store_mut(&mut self) -> &mut ObjectStore<Step, StepId> {
    &mut self.step_store
  }

  pub fn push_root_substep(&mut self, step_id: StepId) {
    let root_step = self.step_store.get_mut(&self.step_id_root).unwrap();
    root_step.push_substep(step_id);
  }

  pub fn action_store(&self) -> &ActionObjectStore {
    &self.action_store
  }

  pub fn varstore(&self) -> &ObjectStore<Box<dyn Var + Sync + Send>, VarId> {
    &self.varstore
  }

  pub fn varstore_mut(&mut self) -> &mut ObjectStore<Box<dyn Var + Sync + Send>, VarId> {
    &mut self.varstore
  }

  /// see if next step will accept with current inputs
  /// if so, advance there (checking for nested states) and return current step
  /// if not, reject and stay on current step (how relay error msg?)
  fn try_enter_next_step(&mut self, step_output: Option<(&StepId, StateData)>)
    -> Result<Option<StepId>, Error>
  {
    if let Some(output) = step_output {
      // make sure we're updating the right state
      if self.current_step()? != output.0 {
        return Err(Error::StepId(IdError::IdUnexpected(output.0.clone())))
      }

      // merge the new inputs in first. best to not lose this even if the rest fails
      self.state_data.merge_from(output.1)
    }

    let state_data = &self.state_data;
    let step_store = &self.step_store;
    self.step_id_dfs.next(
      |step_id| {
        let step = step_store.get(step_id).ok_or_else(|| Error::StepId(IdError::IdMissing(step_id.clone())))?;
        step.can_enter(&state_data).map_err(|e| Error::VarId(e))
      },
      |step_id| {
        let step = step_store.get(step_id).ok_or_else(|| Error::StepId(IdError::IdMissing(step_id.clone())))?;
        step.can_exit(&state_data).map_err(|e| Error::VarId(e))
      },
      &self.step_store)
  }

  fn call_action(&mut self, action_id: &ActionId, step_id: &StepId) -> Result<ActionResult, Error> {
    // setup params
    fn get_step_input_output_vars(step: &Step) -> HashSet<VarId> {
      step.get_input_vars()
        .clone()      
        .unwrap_or_else(|| vec![])
        .iter()
        .chain(step.get_output_vars().iter())
        .map(|id_ref| id_ref.clone())
        .collect::<HashSet<VarId>>()
    }
  
    let step = self.step_store.get(step_id).ok_or_else(|| Error::StepId(IdError::IdMissing(step_id.clone())))?;
    let step_name = self.step_store.name_from_id(&step_id);
    let step_data: StateDataFiltered = StateDataFiltered::new(&self.state_data, get_step_input_output_vars(&step));
    let vars = ObjectStoreFiltered::new(&self.varstore, get_step_input_output_vars(&step));

    // call it
    let action_result = self.action_store.start_action(action_id, &step, step_name, &step_data, &vars)?;
    match &action_result {
        ActionResult::Finished(state_data) => {
          if !state_data.contains_only(&step.output_vars.iter().collect::<HashSet<_>>()) {
            return Err(Error::InvalidStateDataError);
          }
        }
        ActionResult::StartWith(_) |
        ActionResult::CannotFulfill => ()
    }
    Ok(action_result)
  }  

  // we never allow an execute-on-current-state because we may be in a non-ready/transitional state (i.e. first state) and we have to advance first
  // we always try to execute, even if we can't advance on the hopes that executing again may work the next time (i.e. form field with invalid fields)
  pub fn advance(&mut self, step_output: Option<(&StepId, StateData)>) 
      -> Result<AdvanceBlockedOn, Error>
  {
    #[derive(Clone)]
    enum States {
      AdvanceStep,
      GetSpecificAction(StepId, Option<Error>),  // current step id, step-id-advance error
      GetGenericAction(StepId, Option<Error>),      // step-id-advance error
      StartSpecific(ActionId, StepId, Option<Error>), // action id, step-id-advance error
      StartGeneric(ActionId, StepId, Option<Error>),  // action id, step-id-advance error
      Done(Result<AdvanceBlockedOn, Error>)
    }

    // generally we're trying to advance as much as possible without user interaction:
    // loop until we get to a blocking state (StartWith or No-more-states-left or can't-start)
    //   advance step
    //   succeed or fail:
    //     start specific action
    //     if doesn't exist or succeed, start generic action
    // return (step-advance-result, action-result)
    let mut step_output = step_output;
    let mut state = States::AdvanceStep;
    loop {
      /*
      if let States::Done(result) = state {
        return result;
      }
      */
      state = match state.clone() {
        States::Done(result) => return result,
        States::AdvanceStep => {
          let advance_result = self.try_enter_next_step(step_output);
          step_output = None;
          match &advance_result {
            Ok(step_id_opt) => {
              match step_id_opt {
                Some(step_id) => States::GetSpecificAction(step_id.clone(), None),
                None => States::Done(Ok(AdvanceBlockedOn::FinishedAdvancing)), // no more steps left to advance
              }
            }
            Err(err) => {
              let step_id = self.current_step()?.clone();
              States::GetSpecificAction(step_id, Some(err.clone())) // error advancing but we can try the action to see if that fixes it
            }
          }
        },
        States::GetSpecificAction(step_id, error) => {
          match self.step_actions.get(&step_id) {
            Some(action_id) => States::StartSpecific(action_id.clone(), step_id, error),
            None => States::GetGenericAction(step_id, error),
          }
        },
        States::GetGenericAction(step_id, error) => {
          match self.step_actions.get(&self.step_id_all) {
            Some(action_id) => States::StartGeneric(action_id.clone(), step_id, error),
            None => {
              match error {
                None => States::AdvanceStep,  // did we advance? if so, try advancing again
                Some(err) => return Err(err),   // couldn't advance and no action? then we're stuck
              }
            }
          }
        },
        States::StartSpecific(action_id, step_id, error_opt) |
        States::StartGeneric(action_id, step_id, error_opt) => {
          let action_result = self.call_action(&action_id, &step_id)?;
          match action_result {
              ActionResult::StartWith(val) => {
                States::Done(Ok(AdvanceBlockedOn::ActionStartWith(action_id, val)))
              }
              ActionResult::Finished(state_data) => {
                // merge the new data and see if we can keep advancing
                self.state_data.merge_from(state_data.clone());
                States::AdvanceStep
              }
              ActionResult::CannotFulfill => {
                if matches!(state, States::StartSpecific(_,_,_)) {
                  // couldn't fulfill specific action, try generic one
                  States::GetGenericAction(step_id, error_opt)
                } else {
                  // couldn't fulfill generic one (and must've already failed specific) -- nothing else we can do
                  States::Done(Ok(AdvanceBlockedOn::ActionCannotFulfill))
                }
              }
          }
        }
      }
    }
  }

  pub fn set_action_for_step(&mut self, action_id: ActionId, step_id:Option<&StepId>) 
      -> Result<(), Error>
  {
    let step_id_use = step_id.or(Some(&self.step_id_all)).unwrap();
    if self.step_actions.contains_key(step_id_use) {
      return Err(Error::StepId(IdError::IdAlreadyExists(step_id_use.clone())));
    }
    self.step_actions.insert(step_id_use.clone(), action_id);
    Ok(())
  }

  #[cfg(test)]
  pub fn test_new() -> (Session, StepId) {
    let mut session = Session::new(stepflow_test_util::test_id!(SessionId));
    let root_step_id = session.step_store_mut().insert_new(Some("root_step".to_owned()), |id| Ok(Step::new(id, None, vec![]))).unwrap();
    session.push_root_substep(root_step_id.clone());
    (session, root_step_id)
  }

  #[cfg(test)]
  pub fn test_new_stringvar(&mut self) -> VarId {
    let var_id = stepflow_test_util::test_id!(VarId);
    let var = stepflow_data::StringVar::new(var_id);
    let var_id = self.varstore.register(None, var.boxed()).unwrap();
    var_id
  }
}

#[derive(Debug, Clone)]
pub enum AdvanceBlockedOn {
  ActionStartWith(ActionId, Box<dyn Value>),
  ActionCannotFulfill,
  FinishedAdvancing,
}

impl PartialEq for AdvanceBlockedOn {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (AdvanceBlockedOn::ActionStartWith(action_id, val),AdvanceBlockedOn::ActionStartWith(action_id_other, val_other)) => {
        action_id == action_id_other && val == val_other
      }
      (AdvanceBlockedOn::ActionCannotFulfill, AdvanceBlockedOn::ActionCannotFulfill) |
      (AdvanceBlockedOn::FinishedAdvancing, AdvanceBlockedOn::FinishedAdvancing) => {
        true
      }
      _ => false
    }
  }
}


#[cfg(test)]
mod tests {
  use core::panic;
  use stepflow_base::{ObjectStore, IdError};
  use stepflow_data::{BoolValue, StateData, VarId, StringValue};
  use stepflow_step::{Step, StepId};
  use stepflow_test_util::test_id;
  use stepflow_action::{SetDataAction, Action, ActionId};
  use crate::test::TestAction;
  use super::super::{Error};
  use super::{Session, SessionId, AdvanceBlockedOn};



  fn new_simple_step(id: StepId) -> Result<Step, IdError<StepId>> {
    Ok(Step::new(id, None, vec![]))
  }

  fn add_new_simple_substep(parent_id: &StepId, step_store: &mut ObjectStore<Step, StepId>) -> StepId {
    let substep_id = step_store.insert_new(None, new_simple_step).unwrap();
    push_substep(parent_id, substep_id, step_store)
  }

  fn push_substep(parent_id: &StepId, step_id: StepId, step_store: &mut ObjectStore<Step, StepId>) -> StepId {
    let parent = step_store.get_mut(parent_id).unwrap();
    parent.push_substep(step_id.clone());
    step_id
  }

  fn step_str_output(session: &Session, var_id: &VarId, val: &str) -> (StepId, StateData) {
    let mut state_data = StateData::new();
    let var = session.varstore().get(var_id).unwrap();
    state_data.insert(var, StringValue::try_new(val.to_owned()).unwrap().boxed()).unwrap();
    (session.current_step().unwrap().clone(), state_data)
  }

  #[test]
  fn empty_session_advance() {
    let mut session = Session::new(test_id!(SessionId));
    let advance_result = session.advance(None);
    assert_eq!(advance_result, Ok(AdvanceBlockedOn::FinishedAdvancing));
  }

  #[test]
  fn progress_session_inputs_outputs() {
    let mut session = Session::new(test_id!(SessionId));

    let var_output1_id = session.test_new_stringvar();
    let var_input2_id = session.test_new_stringvar();
    let var_output2_id = session.test_new_stringvar();

    let root_step_id = session.step_store.insert_new(
      Some("root_step".to_owned()), |id| {
        Ok(Step::new(
          id,
          Some(vec![var_input2_id.clone()]),
          vec![var_output1_id.clone(), var_output2_id.clone()]))
      })
      .unwrap();
    session.push_root_substep(root_step_id);
    
    let substep1_id = session.step_store_mut().insert_new(
      Some("SubStep 1".to_owned()),
      |id| Ok(Step::new(id, None, vec![var_output1_id.clone()])))
      .unwrap();
    let substep2_id = session.step_store_mut().insert_new(
      Some("SubStep 2".to_owned()),
      |id| Ok(Step::new(id, Some(vec![var_input2_id.clone()]), vec![var_output2_id.clone()])))
      .unwrap();

    let root_step = session.step_store_mut().get_mut(&root_step_id).unwrap();
    root_step.push_substep(substep1_id.clone());
    root_step.push_substep(substep2_id.clone());
    
    assert_eq!(session.try_enter_next_step(None), Err(Error::VarId(IdError::IdMissing(var_input2_id.clone()))));    // start without proper input

    // go to substep1
    let output1 = step_str_output(&session, &var_input2_id, "input2");
    assert_eq!(session.try_enter_next_step(Some((&output1.0, output1.1))), Ok(Some(substep1_id.clone())));  // start without proper input

    // go to substep2
    assert_eq!(session.try_enter_next_step(None), Err(Error::VarId(IdError::IdMissing(var_output1_id.clone()))));  // didn't add output
    let output2 = step_str_output(&session, &var_output1_id, "output1");
    assert_eq!(session.try_enter_next_step(Some((&output2.0, output2.1))), Ok(Some(substep2_id.clone())));

    // done with states but can't leave root without the output from substep 2
    assert_eq!(session.try_enter_next_step(None), Err(Error::VarId(IdError::IdMissing(var_output2_id.clone()))));
    let output3 = step_str_output(&session, &var_output2_id, "output2");
    assert_eq!(session.try_enter_next_step(Some((&output3.0, output3.1))), Ok(None));
    
    // try it again to check we're still done advancing
    assert_eq!(session.try_enter_next_step(None), Ok(None));
  }

  #[test]
  fn simple_action() {
    let (mut session, root_step_id) = Session::test_new();

    let substep1 = add_new_simple_substep(&root_step_id, session.step_store_mut());
    let substep2 = add_new_simple_substep(&root_step_id, session.step_store_mut());
    let substep3 = add_new_simple_substep(&root_step_id, session.step_store_mut());

    let test_action_id = session.action_store().insert_new(
      None, 
      |id| Ok(TestAction::new_with_id(id, true).boxed()))
      .unwrap();
    session.set_action_for_step(test_action_id, None).unwrap();

    let mut steps_executed:Vec<StepId> = vec![];
    loop {
      match session.advance(None) {
        Ok(advance_result) => {
          match advance_result {
            AdvanceBlockedOn::ActionStartWith(_, _) => (),
            AdvanceBlockedOn::FinishedAdvancing => break,
            _ => panic!("Unexpected advance result: {:?}", advance_result),
          }
        },
        Err(err) => {
          panic!("unexpected error trying to advance: {:?}", err);
        },
      }
      steps_executed.push(session.current_step().unwrap().clone());
    }

    // make sure we advanced all the steps
    assert_eq!(steps_executed, vec![substep1, substep2, substep3]);
  }


  #[test]
  fn specific_generic_actions() {

    // create session + steps
    let (mut session, root_step_id) = Session::test_new();
    let var_id = session.test_new_stringvar();

    let substep1 = session.step_store_mut().insert_new(None, |id| {
        Ok(Step::new(id, None, vec![var_id.clone()]))
      })
      .unwrap();
    push_substep(&root_step_id, substep1.clone(), session.step_store_mut());
    
    let substep2 = session.step_store_mut().insert_new(
      None,
      |id| Ok(Step::new(id, Some(vec![var_id.clone()]), vec![var_id.clone()])))
      .unwrap();
    push_substep(&root_step_id, substep2.clone(), session.step_store_mut());

    // create statedata for action
    let mut statedata_exec = StateData::new();
    let var = session.varstore().get(&var_id).unwrap();
    statedata_exec.insert(var, StringValue::try_new("hi".to_owned()).unwrap().boxed()).unwrap();

    // create actions
    let set_action_id = session.action_store().insert_new(None, |id| {
      Ok(SetDataAction::new(id, statedata_exec, 2).boxed())
    }).unwrap();

    let test_action_id = session.action_store().insert_new(None, |id| {
        Ok(TestAction::new_with_id(id, true).boxed())
      })
      .unwrap();

    // set action for substep1, test_action as generic one
    session.set_action_for_step(set_action_id, Some(&substep1)).unwrap();
    session.set_action_for_step(test_action_id, None).unwrap();

    // 1. advance to substep 1, fail to execute specific setval, succeed generic test_action
    if let AdvanceBlockedOn::ActionStartWith(_, _) = session.advance(None).unwrap() {
      assert_eq!(*session.current_step().unwrap(), substep1.clone()); // advanced to substep1
    } else {
      panic!("did not advance");
    }

    // 2. fail advance to substep2 (setval::count=1 now but min is 2), succeed setval::count=2
    if let AdvanceBlockedOn::ActionStartWith(_, _) = session.advance(None).unwrap() {
      assert!(!session.state_data.contains(&var_id)); // setval still hasn't worked
    } else {
      panic!("did not advance");
    }

    // 3. succeed advance to substep2 (setval executed, then advanced step), succeed generic test_action
    if let AdvanceBlockedOn::ActionStartWith(_, _) = session.advance(None).unwrap() {
      assert_eq!(*session.current_step().unwrap(), substep2.clone()); // advanced to substep2
      assert!(session.state_data.contains(&var_id)); // setval worked
    } else {
      panic!("did not advance");
    }

    // 4. done
    assert_eq!(
      session.advance(None).unwrap(),
      AdvanceBlockedOn::FinishedAdvancing);
  }

  #[test]
  fn auto_advance() {
    let (mut session, root_step_id) = Session::test_new();
    let test_action_id = session.action_store().insert_new(None, |id| {
        Ok(TestAction::new_with_id(id, false).boxed())
      })
      .unwrap();

    let _substep1 = add_new_simple_substep(&root_step_id, session.step_store_mut());
    let _substep2 = add_new_simple_substep(&root_step_id, session.step_store_mut());
    let _substep3 = add_new_simple_substep(&root_step_id, session.step_store_mut());
    
    session.set_action_for_step(test_action_id, None).unwrap();

    // one call should advance to the end as we test_action keeps finishing so can keep advancing
    let advance = session.advance(None);
    assert_eq!(advance, Ok(AdvanceBlockedOn::FinishedAdvancing));
  }

  #[test]
  fn advance_blocked_on_eq() {
    let abo_finish = AdvanceBlockedOn::FinishedAdvancing;
    assert_eq!(abo_finish, abo_finish);

    let abo_cannot_fulfill = AdvanceBlockedOn::ActionCannotFulfill;
    assert_ne!(abo_finish, abo_cannot_fulfill);

    let action_id = test_id!(ActionId);
    let abo_start_true = AdvanceBlockedOn::ActionStartWith(action_id.clone(), BoolValue::new(true).boxed());
    let abo_start_false = AdvanceBlockedOn::ActionStartWith(action_id, BoolValue::new(false).boxed());
    assert_eq!(abo_start_false, abo_start_false);
    assert_ne!(abo_start_true, abo_start_false);
    assert_ne!(abo_start_false, abo_finish);
  }

}

