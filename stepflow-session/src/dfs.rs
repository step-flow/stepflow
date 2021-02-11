use stepflow_base::ObjectStore;
use stepflow_step::{Step, StepId};
use super::{Error};

#[derive(PartialEq, Clone, Debug)]
enum DFSDirection {
  Down,
  SiblingOrUp,
  Done,
}

#[derive(Debug)]
enum DFSStep {
  DownTo(StepId),
  SiblingTo(StepId),
  CannotGoto(Error),
  CannotLeaveForSibling(Error),
  NothingMoreDown,
  NothingMoreInStack,
  PoppedUp,
}

#[derive(Debug)]
pub struct DepthFirstSearch {
  stack: Vec<StepId>,
  next_direction: DFSDirection,
}

impl DepthFirstSearch {
  pub fn new(root: StepId) -> Self {
    DepthFirstSearch {
      stack: vec![root],
      next_direction: DFSDirection::Down,
    }
  }

  pub fn current(&self) -> Option<&StepId> {
    self.stack.last()
  }

  fn next_sibling_of_current<'store>(&self, step_store: &'store ObjectStore<Step, StepId>) -> Option<&'store StepId> {
    let stack_len = self.stack.len();
    if stack_len < 2 {
      return None;
    }
    let current_id = self.stack.get(stack_len - 1).unwrap();
    let parent_id = self.stack.get(stack_len - 2).unwrap();
    let parent_step = step_store.get(parent_id)?;
    parent_step.next_substep(current_id)
  }

  fn first_child_of<'stateid, 'store>(&self, step_id: &'stateid StepId, step_store: &'store ObjectStore<Step, StepId>) -> Option<&'store StepId> {
    let step = step_store.get(step_id)?;
    step.first_substep()
  }

  fn go_down<FnCanEnter>(&mut self, mut can_enter: FnCanEnter, step_store: &ObjectStore<Step, StepId>) -> DFSStep 
      where FnCanEnter: FnMut(&StepId) -> Result<(), Error>
  {
    // get current node (top of stack)
    let step_id_option = self.stack.last();
    if step_id_option.is_none() {
      return DFSStep::NothingMoreInStack;
    }
    let step_id = step_id_option.unwrap();

    // go to its first child
    match self.first_child_of(step_id, step_store) {
      Some(first_child) => {
        if let Err(e) = can_enter(&first_child) {
          return DFSStep::CannotGoto(e);
        }
        self.stack.push(first_child.clone());
        DFSStep::DownTo(first_child.clone())
      },
      None => DFSStep::NothingMoreDown,
    }
  }

  fn go_sibling_or_up<FnCanEnter, FnCanExit>(&mut self, can_enter: &mut FnCanEnter, mut can_exit: FnCanExit, step_store: &ObjectStore<Step, StepId>) -> DFSStep 
      where FnCanEnter: FnMut(&StepId) -> Result<(), Error>,
            FnCanExit: FnMut(&StepId) -> Result<(), Error>
  {
    // get current node (top of the stack)
    let top_stack = self.stack.last();
    if top_stack.is_none() {
      return DFSStep::NothingMoreInStack;
    }

    // see if we can exit it
    if let Err(e) = can_exit(top_stack.as_ref().unwrap()) {
      return DFSStep::CannotLeaveForSibling(e);
    }

    match self.next_sibling_of_current(step_store) {
      Some(next_sibling) => {
        if let Err(e) = can_enter(next_sibling) {
          return DFSStep::CannotGoto(e);
        }
        self.stack.pop();
        self.stack.push(next_sibling.clone());
        DFSStep::SiblingTo(next_sibling.clone())
      },
      None => {
        self.stack.pop();
        DFSStep::PoppedUp
      }
    }
  }

  pub fn next<FnCanEnter, FnCanExit>(&mut self, mut can_enter: FnCanEnter, mut can_exit: FnCanExit, step_store: &ObjectStore<Step, StepId>)
      -> Result<Option<StepId>, Error> 
      where FnCanEnter: FnMut(&StepId) -> Result<(), Error>,
            FnCanExit: FnMut(&StepId) -> Result<(), Error>
  {
    let mut next_direction = self.next_direction.clone();
    let mut err: Option<Error> = None;
    while err == None {
      let step_result = match next_direction {
        DFSDirection::Down => self.go_down(&mut can_enter, step_store),
        DFSDirection::SiblingOrUp => self.go_sibling_or_up(&mut can_enter, &mut can_exit, step_store),
        DFSDirection::Done => DFSStep::NothingMoreInStack,
      };

      next_direction = match step_result {
        // we found something so continue downward to deepest point
        DFSStep::DownTo(_to_step_id) => DFSDirection::Down,  // keep going downward
        DFSStep::SiblingTo(_to_sibling) => DFSDirection::Down, // went to a sibling, now go to it's deepest child

        // we've gone down as far as we could, return what we have and next time move to the sibling
        DFSStep::NothingMoreDown => {
          next_direction = DFSDirection::SiblingOrUp;
          break;
        },

        // we've hit the end of the siblings and popped up one, now go to the next sibling
        DFSStep::PoppedUp => DFSDirection::SiblingOrUp,

        // handle various error states
        DFSStep::CannotGoto(step_err) |
        DFSStep::CannotLeaveForSibling(step_err) => {
          // direction stays the same
          err = Some(step_err);
          next_direction
        },
        DFSStep::NothingMoreInStack => {
          next_direction = DFSDirection::Done;
          break;
        },
      }
    }
    self.next_direction = next_direction;
    if let Some(e) = err {
      Err(e)
    } else if self.next_direction == DFSDirection::Done {
      Ok(None)
    } else {
      self.stack.last().map(|stack_id| Some(stack_id.clone())).ok_or(Error::NoStateToEval)
    }
  }
}

#[cfg(test)]
mod tests {
  use stepflow_base::ObjectStore;
  use stepflow_step::{Step, StepId};
  use super::{DepthFirstSearch, Error};

  fn check_fail(fail: Option<&StepId>, step_id_check: &StepId, has_failed: &mut bool) -> Result<(), Error> {
    if *has_failed {
      return Ok(());
    }
    if let Some(step_id_fail) = fail {
      if step_id_fail == step_id_check {
        *has_failed = true;
        return Err(Error::InvalidStateDataError)
      }
    }
    Ok(())
  }

  fn assert_dfs_order(root: StepId, step_store: &ObjectStore<Step, StepId>, expected_children: &Vec<StepId>, fail_on_enter: Option<&StepId>, fail_on_exit: Option<&StepId>) {
    let mut dfs = DepthFirstSearch::new(root);
    let mut count_matches = 0;
    let mut failed_enter = false;
    let mut failed_exit = false;
    let mut expected_iter = expected_children.iter();
    let mut expected_child_opt = expected_iter.next();
    loop {
      // get next expected child
      if expected_child_opt.is_none() {
        break;
      }
      let expected_child = expected_child_opt.unwrap();

      // get next child from DFS
      let next = dfs.next(|step_id: &StepId| {
          check_fail(fail_on_enter, step_id, &mut failed_enter)
        },
        |step_id: &StepId| {
          check_fail(fail_on_exit, step_id, &mut failed_exit)
        },
        step_store);

      // handle result
      match next {
        Ok(step_id_opt) => {
          if let Some(step_id) = step_id_opt {
            if step_id != *expected_child {
              break;
            } else {
              count_matches = count_matches + 1;
              expected_child_opt = expected_iter.next();
            }
          } else {
            // we're done with the entire tree
            break;
          }
        },
        Err(err) => {
          // make sure it came from check_fail
          assert_eq!(err, Error::InvalidStateDataError);
        }
      }
    }
    assert_eq!(count_matches, expected_children.len());

    // we may be on the last step and asked to fail it's exit so we allow for
    // 2 passes on the final next() call for this situation (the first pass generates an error from the forced failure)
    for pass in 0..1 {
      let final_next = dfs.next(|step_id: &StepId| {
          check_fail(fail_on_enter, step_id, &mut failed_enter)
        },
        |step_id: &StepId| {
          check_fail(fail_on_exit, step_id, &mut failed_exit)
        },
        step_store);

      match final_next {
        Ok(step_id_opt) => assert_eq!(step_id_opt, None),
        Err(err) => {
          assert_eq!(pass, 0);  // we only force-fail once so it should only happen on the first pass
          assert_eq!(err, Error::InvalidStateDataError);
        }
      }
    }

    // make sure we failed something if we're testing for it
    if fail_on_enter.is_some() {
      assert_eq!(failed_enter, true);
    }
    if fail_on_exit.is_some() {
      assert_eq!(failed_exit, true);
    }
  }

  fn assert_dfs_order_with_failures(root: StepId, step_store: &ObjectStore<Step, StepId>, expected_children: &Vec<StepId>) {
    assert_dfs_order(root.clone(), step_store, expected_children, None, None);
    for ienter in 0..expected_children.len() {
      for iexit in 0..expected_children.len() {
        assert_dfs_order(root.clone(), step_store, expected_children, Some(&expected_children[ienter]), Some(&expected_children[iexit]));
      }
    }
  }

  fn add_substeps(num: usize, parent_id: &StepId, step_store: &mut ObjectStore<Step, StepId>) -> Vec<StepId> {
    let mut result = Vec::new();
    for _ in 0..num {
      let substep_id = step_store.insert_new(|id| Ok(Step::new(id, None, vec![]))).unwrap();
      let parent_step = step_store.get_mut(parent_id).unwrap();
      parent_step.push_substep(substep_id.clone());
      result.push(substep_id);
    }
    result
  }

  #[test]
  fn one_deep() {
    let mut step_store: ObjectStore<Step, StepId> = ObjectStore::new();
    let root = step_store.insert_new(|id| Ok(Step::new(id, None, vec![]))).unwrap();
    let child_ids = add_substeps(2, &root, &mut step_store);
    assert_dfs_order_with_failures(root, &step_store, &child_ids);
  }

  #[test]
  fn two_deep() {
    let mut step_store: ObjectStore<Step, StepId> = ObjectStore::new();
    let root = step_store.insert_new(|id| Ok(Step::new(id, None, vec![]))).unwrap();
    let root_children = add_substeps(2, &root, &mut step_store);
    let children_1 = add_substeps(3, &root_children[0], &mut step_store);
    let children_2 = add_substeps(3, &root_children[1], &mut step_store);

    let mut expected_children = Vec::new();
    expected_children.extend(children_1);
    expected_children.extend(children_2);
    assert_dfs_order_with_failures(root, &step_store, &expected_children);
  }

  #[test]
  fn mixed_depth() {
    let mut step_store: ObjectStore<Step, StepId> = ObjectStore::new();
    let root = step_store.insert_new(|id| Ok(Step::new(id, None, vec![]))).unwrap();
    let root_children = add_substeps(3, &root, &mut step_store);
    let children1 = add_substeps(1, &root_children[0].clone(), &mut step_store);
    let children3 = add_substeps(3, &root_children[2].clone(), &mut step_store);
    let children3_children2 = add_substeps(3, &children3[1].clone(), &mut step_store);

    let mut expected_children = Vec::new();
    expected_children.extend(children1);
    expected_children.push(root_children[1].clone());
    expected_children.push(children3[0].clone());
    expected_children.extend(children3_children2);
    expected_children.push(children3[2].clone());

    assert_dfs_order_with_failures(root, &step_store, &expected_children);
  }
}