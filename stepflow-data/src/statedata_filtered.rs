use std::collections::HashSet;
use crate::{StateData, VarId, ValidVal};

pub struct StateDataFiltered<'sd> {
  allowed_var_ids: HashSet<VarId>,
  state_data: &'sd StateData,
}

impl<'sd> StateDataFiltered<'sd> {
  pub fn new(state_data: &'sd StateData, allowed_var_ids: HashSet<VarId>) -> Self {
    Self { state_data, allowed_var_ids }
  }

  pub fn get(&self, var_id: &VarId) -> Option<&ValidVal> {
    if !self.allowed_var_ids.contains(var_id) {
      return None
    }
    self.state_data.get(var_id)
  }

  pub fn contains(&self, var_id: &VarId) -> bool {
    if !self.allowed_var_ids.contains(var_id) {
      return false;
    }
    self.state_data.contains(var_id)
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashSet;
  use crate::{StateData, ValidVal, test_var_val};
  use super::StateDataFiltered;

  #[test]
  fn basic() {
    let var1 = test_var_val();
    let var2 = test_var_val();

    let val1_valid = ValidVal::try_new(var1.1.clone(), &var1.0).unwrap();

    // add var1 + var2
    let mut data = StateData::new();
    data.insert(&var1.0, var1.1).unwrap();
    data.insert(&var2.0, var2.1).unwrap();

    // create filtered statedata
    let mut filter = HashSet::new();
    filter.insert(var1.0.id().clone());
    let data_filtered = StateDataFiltered::new(&data, filter);

    assert_eq!(data_filtered.get(var1.0.id()), Some(&val1_valid));
    assert_eq!(data_filtered.get(var2.0.id()), None);
  }

}
