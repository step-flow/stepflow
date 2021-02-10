use std::collections::{HashMap, HashSet};
use super::{InvalidValue, InvalidVars};
use super::value::{Value, ValidVal};
use super::var::{Var, VarId};

/// Store a set of [`Var`]s and corresponding [`Value`]s.
///
/// Internally the [`Value`] is wrapped in a [`ValidVal`](crate::value::ValidVal) to keep knowledge that this value has been validated for a specific [`Var`] already.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct StateData {
  data: HashMap<VarId, ValidVal>,
}

impl StateData {
  /// Create a new StateData instance
  pub fn new() -> Self {
    Self {
      data: HashMap::new()
    }
  }

  /// Add a new value
  pub fn insert(&mut self, var: &Box<dyn Var + Send + Sync>, state_val: Box<dyn Value>)  -> Result<(), InvalidValue> {
    let state_val_valid = ValidVal::try_new(state_val, var)?;
    self.data.insert(var.id().clone(), state_val_valid);
    Ok(())
  }

  /// Get the value based on its [`VarId`]. Returns a [`ValidVal`] to keep knowledge that the value has already been validated for the specific [`Var`].
  pub fn get(&self, var_id: &VarId) -> Option<&ValidVal> {
    self.data.get(var_id)
  }

  pub fn contains(&self, var_id: &VarId) -> bool {
    self.data.contains_key(var_id)
  }

  /// Confirm that the StateData *only* contains the set of [`VarId`]s listed
  pub fn contains_only(&self, contains_only: &HashSet<&VarId>) -> bool {
    let found_excluded = self.data.iter().find(|(var_id, _)| !contains_only.contains(var_id));
    found_excluded == None
  }

  /// Merge the data from another `StateData` into this one.
  pub fn merge_from(&mut self, src: StateData) {
    for (k, v) in src.data {
      self.data.insert(k, v);
    }
  }

  // Get an iterator over the values
  pub fn iter_val(&self) -> impl Iterator<Item = (&VarId, &Box<dyn Value>)>  {
    self.data.iter().map(|(var_id, valid_val)| {
      (var_id, valid_val.get_val())
    })
  }


  /// Create a `StateData` instance from an iterator of values
  // NOTE: can't implement TryFrom for this because of blanket implementation in core
  pub fn from_vals<'a, T>(iter: T)  -> Result<Self, InvalidVars> 
    where T : std::iter::IntoIterator<Item = (&'a Box<dyn Var + Send + Sync + 'static>, Box<dyn Value>)>
  {
    let mut all_valid = true;
    let validations = iter.into_iter()
      .map(|(var, val)| {
        match ValidVal::try_new(val, var) {
          Ok(validated) => Ok((var, validated)),
          Err(e) => {
            all_valid = false;
            Err((var, e))
          }
        }
      })
      .collect::<Vec<Result<_,_>>>();

    if !all_valid {
      let invalid: HashMap<VarId, InvalidValue> = validations.into_iter().filter_map(|validation| {
        if let Err(e) = validation {
          Some((e.0.id().clone(), e.1))
        } else {
          None
        }
      })
      .collect();
      return Err(InvalidVars::new(invalid));
    }

    let data: HashMap<VarId, ValidVal> = validations
      .into_iter()
      .map(|validation| {
        let valid = validation.unwrap();
        (valid.0.id().clone(), valid.1)
      })
      .collect();
    Ok(StateData { data })
  }
}


#[cfg(test)]
mod tests {
  use std::collections::{HashMap, HashSet};
  use crate::{var::{Var, VarId, StringVar}, value::{Value, TrueValue}, InvalidValue, test_var_val};
  use stepflow_test_util::test_id;
  use super::{StateData, InvalidVars};

  #[test]
  fn merge() {
    let mut data1 = StateData::new();
    let mut data2 = StateData::new();
    let mut data_merged = StateData::new();

    let var1 = test_var_val();
    let var2 = test_var_val();
    let var3 = test_var_val();
    let var4 = test_var_val();

    data1.insert(&var1.0, var1.1).unwrap();
    data2.insert(&var2.0, var2.1).unwrap();
    data2.insert(&var3.0, var3.1).unwrap();
    data_merged.insert(&var4.0, var4.1).unwrap();

    assert!(!data_merged.contains(var1.0.id()));
    data_merged.merge_from(data1);
    assert!(data_merged.contains(var1.0.id()));

    assert!(!data_merged.contains(var2.0.id()));
    assert!(!data_merged.contains(var3.0.id()));
    data_merged.merge_from(data2);
    assert!(data_merged.contains(var2.0.id()));
    assert!(data_merged.contains(var3.0.id()));
  }

  #[test]
  fn from_vals_err() {
    let var1 = test_var_val();
    let var2 = test_var_val();
    let badvar1: (Box<dyn Var + Send + Sync>, Box<dyn Value>) = (
      Box::new(StringVar::new(test_id!(VarId))),
      Box::new(TrueValue::new()));
    let badvar2: (Box<dyn Var + Send + Sync>, Box<dyn Value>) = (
      Box::new(StringVar::new(test_id!(VarId))),
      Box::new(TrueValue::new()));
    let badvar1_id = badvar1.0.id().clone();
    let badvar2_id = badvar2.0.id().clone();

    let vars = vec![var1, badvar1, var2, badvar2];
    let vars = vars
      .iter()
      .map(|(var, val)| {
        (var, val.clone())
      });

    let mut bad_ids = HashMap::new();
    bad_ids.insert(badvar1_id.clone(), InvalidValue::WrongType);
    bad_ids.insert(badvar2_id.clone(), InvalidValue::WrongType);
    let expected_err = InvalidVars(bad_ids);

    assert_eq!(StateData::from_vals(vars), Err(expected_err));
  }

  #[test]
  fn contains_only() {
    let mut data = StateData::new();

    let var1 = test_var_val();
    let var2 = test_var_val();
    let var3 = test_var_val();

    // add var1 + var2
    data.insert(&var1.0, var1.1).unwrap();
    data.insert(&var2.0, var2.1).unwrap();

    let mut contains_only = HashSet::new();
    contains_only.insert(var1.0.id());
    contains_only.insert(var2.0.id());

    // check only contains var1 + var2
    assert_eq!(data.contains_only(&contains_only), true);

    // add var3
    data.insert(&var3.0, var3.1).unwrap();

    // check only contains var1 + var2
    assert!(!data.contains_only(&contains_only));
  }

  #[test]
  fn iter() {
    let mut data = StateData::new();
    let var1 = test_var_val();
    let var2 = test_var_val();
    data.insert(&var1.0, var1.1.clone()).unwrap();
    data.insert(&var2.0, var2.1.clone()).unwrap();

    let hashmap = data.iter_val().collect::<HashMap<_,_>>();
    assert_eq!(hashmap.len(), 2);
    assert_eq!(hashmap.get(var1.0.id()), Some(&&var1.1));
    assert_eq!(hashmap.get(var2.0.id()), Some(&&var2.1));
  }
}