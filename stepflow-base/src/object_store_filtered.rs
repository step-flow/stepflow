use std::hash::Hash;
use std::collections::HashSet;
use crate::{ObjectStore, ObjectStoreContent};

/// Wrapper to an [`ObjectStore`](crate::ObjectStore) that provides a filtered view of the objects contained
pub struct ObjectStoreFiltered<'os, T, TID> 
  where TID: Eq + Hash + 'static
{
  allowed_ids: HashSet<TID>,
  object_store: &'os ObjectStore<T, TID>,
}

impl<'os, T, TID> ObjectStoreFiltered<'os, T, TID>
  where T:ObjectStoreContent + ObjectStoreContent<IdType = TID>,
  TID: Eq + Hash + Clone + 'static,
{
  /// Wrap the `object_store` with a filtered view. Only IDs specified in `allowed_ids` are visible.
  pub fn new(object_store: &'os ObjectStore<T, TID>, allowed_ids: HashSet<TID>) -> Self {
    Self { allowed_ids, object_store }
  }

  pub fn id_from_name(&self, name: &str) -> Option<&TID> {
    if let Some(id) = self.object_store.id_from_name(name) {
      if self.allowed_ids.contains(id) {
        return Some(id);
      }
    }
    None
  }

  pub fn name_from_id(&self, id: &TID) -> Option<&String> {
    if !self.allowed_ids.contains(id) {
      return None;
    }
    self.object_store.name_from_id(id)
  }

  pub fn get_by_name(&self, name: &str) -> Option<&T> {
    self.id_from_name(name).and_then(|id| self.get(id))
  }

  pub fn get(&self, id: &TID) -> Option<&T> {
    if !self.allowed_ids.contains(id) {
      return None;
    }
    self.object_store.get(id)
  }
}


#[cfg(test)]
mod tests {
  use std::collections::HashSet;
  use crate::{test::TestObject, test::TestObjectId, ObjectStore};
  use super::ObjectStoreFiltered;

  
  #[test]
  fn basic() {
    let mut object_store: ObjectStore<TestObject, TestObjectId> = ObjectStore::new();
    let t1 = object_store.insert_new(Some("t1".to_owned()), |id| Ok(TestObject::new(id, 100))).unwrap();
    let t2 = object_store.insert_new(Some("t2".to_owned()), |id| Ok(TestObject::new(id, 200))).unwrap();

    // create filtered store
    let mut filter = HashSet::new();
    filter.insert(t1.clone());
    let filtered = ObjectStoreFiltered::new(&object_store, filter);

    assert_eq!(filtered.id_from_name("t1"), Some(&t1));
    assert_eq!(filtered.id_from_name("t2"), None);

    assert_eq!(filtered.name_from_id(&t1), Some(&"t1".to_owned()));
    assert_eq!(filtered.name_from_id(&t2), None);

    assert!(matches!(filtered.get_by_name("t1"), Some(_)));
    assert_eq!(filtered.get_by_name("t2"), None);

    assert!(matches!(filtered.get(&t1), Some(_)));
    assert_eq!(filtered.get(&t2), None);
  }

}