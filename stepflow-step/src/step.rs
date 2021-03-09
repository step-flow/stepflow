use stepflow_base::{generate_id_type, IdError, ObjectStoreContent};
use stepflow_data::{StateData, var::VarId};

generate_id_type!(StepId);

#[derive(Debug)]
/// A single step in a flow
///
/// A step is defined by its the required inputs to enter the step and the outputs it must fulfill to exit the step.
/// Substeps allow for grouping of steps and are executing in order by default.
pub struct Step {
  pub id: StepId,
  pub input_vars: Option<Vec<VarId>>,
  pub output_vars: Vec<VarId>,

  substep_step_ids: Option<Vec<StepId>>,
}

impl ObjectStoreContent for Step {
    type IdType = StepId;

    fn new_id(id_val: u16) -> Self::IdType {
      StepId::new(id_val)
    }

    fn id(&self) -> &Self::IdType {
      &self.id
    }
}

impl Step {
  /// Create a new step.
  ///
  /// If no inputs are required, pass in `None` for `input_vars`
  pub fn new(id: StepId, input_vars: Option<Vec<VarId>>, output_vars: Vec<VarId>) -> Self {
    Step {
      id,
      input_vars,
      output_vars,
      substep_step_ids: None,
    }
  }

  #[cfg(test)]
  pub fn test_new() -> Self {
    Step::new(stepflow_test_util::test_id!(StepId), None, vec![])
  }

  pub fn get_input_vars(&self) -> &Option<Vec<VarId>> {
    &self.input_vars
  }

  pub fn get_output_vars(&self) -> &Vec<VarId> {
    &self.output_vars
  }

  /// Push a substep to the end of the current sub-steps
  pub fn push_substep(&mut self, substep_step_id: StepId) {
    match &mut self.substep_step_ids {
      None => self.substep_step_ids = Some(vec![substep_step_id]),
      Some(substep_step_ids) => substep_step_ids.push(substep_step_id),
    }
  }

  /// Get the sub-step that directly follows `prev_substep_id`
  pub fn next_substep(&self, prev_substep_id: &StepId) -> Option<&StepId> {
    let mut skipped = false;
    let mut iter = self.substep_step_ids
      .as_ref()?
      .iter()
      .skip_while(|step_id| {
        // find the prev, let it skip once, then stop
        if skipped { 
          return false;
        }
        if *step_id == prev_substep_id {
          skipped = true;
        }
        true
      });
    iter.next()
  }

  pub fn first_substep(&self) -> Option<&StepId> {
    self.substep_step_ids.as_ref()?.first()
  }

  /// Verifies that `inputs` fulfills the required inputs to enter the step
  pub fn can_enter(&self, inputs: &StateData) -> Result<(), IdError<VarId>> {
    // see if we're missing any inputs
    if let Some(input_vars) = &self.input_vars {
      let first_missing_input = input_vars.iter().find(|input_var_id| !inputs.contains(input_var_id));
      if first_missing_input.is_some() {
        return Err(IdError::IdMissing(first_missing_input.unwrap().clone()))
      }
    }

    Ok(())
  }

  /// Verifies that `state_data` fulfills the required outputs to exit the step
  pub fn can_exit(&self, state_data: &StateData) -> Result<(), IdError<VarId>> {
    // see if we're missing any inputs
    self.can_enter(state_data)?;

    // see if we're missing any outputs
    let first_missing_output = &self.output_vars.iter().find(|output_var_id| !state_data.contains(output_var_id));
    if first_missing_output.is_some() {
      return Err(IdError::IdMissing(first_missing_output.unwrap().clone()))
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use stepflow_base::ObjectStoreContent;
  use super::{ Step };

  #[test]
  fn test_add_get_substep() {
    // no substep
    let mut step = Step::test_new();
    assert_eq!(step.first_substep(), None);

    // add one
    let substep1 = Step::test_new();
    step.push_substep(substep1.id().clone());
    assert_eq!(step.first_substep().unwrap(), substep1.id());
    assert_eq!(step.next_substep(&substep1.id()), None);

    // add another
    let substep2 = Step::test_new();
    step.push_substep(substep2.id().clone());
    assert_eq!(step.first_substep().unwrap(), substep1.id());
    assert_eq!(step.next_substep(substep1.id()).unwrap(), substep2.id());
    assert_eq!(step.next_substep(&substep2.id()), None);
  }
}