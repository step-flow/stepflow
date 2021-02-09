use std::hash::Hash;
use std::collections::{HashMap};
use std::sync::atomic::{AtomicU32, Ordering};
use super::IdError;

pub trait ObjectStoreContent {
  type IdType;
  fn new_id(id_val: u32) -> Self::IdType;
  fn id(&self) -> &Self::IdType;
}

/// A store for objects that are weak referenced by an ID and optional name.
///
/// There are two different ways to insert an object.
/// - Use [`insert_new`](ObjectStore::insert_new) which takes a closure that receives the ID for the new object
/// - Get an ID with [`reserve_id`](ObjectStore::reserve_id) and then [`register`](ObjectStore::register) the object with that ID
///
/// # Examples
/// ```
/// # use stepflow_base::{ObjectStore, ObjectStoreContent, IdError, generate_id_type};
/// # generate_id_type!(ObjectId);
/// # struct Object { id: ObjectId }
/// # impl ObjectStoreContent for Object {
/// #   type IdType = ObjectId;
/// #   fn new_id(id_val: u32) -> Self::IdType { ObjectId::new(id_val) }
/// #   fn id(&self) -> &Self::IdType { &self.id }
/// # }
/// // create an ObjectStore with a test object
/// let mut store = ObjectStore::new();
/// let object_id = store.insert_new(Some("test object".to_owned()), |id| Ok(Object { id })).unwrap();
///
/// // get the object either by ID or name
/// let object = store.get(&object_id).unwrap();
/// let object = store.get_by_name("test object").unwrap();
/// ```
#[derive(Debug)]
pub struct ObjectStore<T, TID> 
    where TID: Eq + Hash
{
  id_to_object: HashMap<TID, T>,
  name_to_id: HashMap<String, TID>,
  next_id: AtomicU32,
}

impl<T, TID> ObjectStore<T, TID> 
    where T:ObjectStoreContent + ObjectStoreContent<IdType = TID>,
          TID: Eq + Hash + Clone,
          
{
  /// Create a new ObjectStore
  pub fn new() -> Self {
    Self::with_capacity(0)
  }

  /// Create a new ObjectStore with initial capacity
  pub fn with_capacity(capacity: usize) -> Self {
    Self {
      id_to_object: HashMap::with_capacity(capacity),
      name_to_id: HashMap::with_capacity(capacity),
      next_id: AtomicU32::new(0)
    }
  }

  /// Reserve an ID in the ObjectStore. Generally followed with a call to [`register`](ObjectStore::register) using the ID.
  pub fn reserve_id(&mut self) -> TID {
    T::new_id(self.next_id.fetch_add(1, Ordering::SeqCst))
  }

  /// Registers an object into the ObjectStore
  pub fn register(&mut self, name: Option<String>, object: T) -> Result<TID, IdError<TID>> {
    if self.id_to_object.contains_key(object.id()) {
      // we passed in the ID to use but somehow the id of the object we got back is a dupe
      return Err(IdError::IdAlreadyExists(object.id().clone()))
    } 
    if let Some(name_str) = &name {
      if self.name_to_id.contains_key(name_str) {
        return Err(IdError::NameAlreadyExists(name_str.clone()))
      }
    }

    let object_id = object.id().clone();
    self.id_to_object.insert(object.id().clone(), object);
    if let Some(name_str) = name {
      self.name_to_id.insert(name_str, object_id.clone());
    }

    Ok(object_id)
  }

  /// Reserves an ID and registers the object in a single call. The object created must use the ID given to the closure.
  pub fn insert_new<CB>(&mut self, name: Option<String>, cb: CB) -> Result<TID, IdError<TID>>
      where CB: FnOnce(TID) -> Result<T, IdError<TID>> {
    let id: TID = self.reserve_id();
    let id_clone = id.clone();
    let object = cb(id)?;
    if *object.id() != id_clone {
      return Err(IdError::IdNotReserved(object.id().clone()));
    }
    self.register(name, object)
  }

  /// Get the Object ID from the name
  pub fn id_from_name(&self, name: &str) -> Option<&TID> {
    self.name_to_id
      .get(&name.to_owned())
  }

  /// Get the name from the Object ID
  pub fn name_from_id(&self, id: &TID) -> Option<&String> {
    self.name_to_id.iter()
      .find(|(_iter_name, iter_id)| { *iter_id == id })
      .and_then(|(name, _)| Some(name))
  }

  /// Get an object by its name
  pub fn get_by_name(&self, name: &str) -> Option<&T> {
    self.id_from_name(name).and_then(|id| self.get(id))
  }

  /// Get an object by its ID
  pub fn get(&self, id: &TID) -> Option<&T> {
    self.id_to_object.get(id)
  }

  /// Get a mutable reference to the object
  pub fn get_mut(&mut self, id: &TID) -> Option<&mut T> {
    self.id_to_object.get_mut(id)
  }

  // Iterator for registered object names
  pub fn iter_names(&self) -> impl Iterator<Item = (&String, &TID)> {
    self.name_to_id.iter()
  }
}


#[cfg(test)]
mod tests {
  use stepflow_test_util::test_id;
  use super::{ObjectStore};
  use crate::{test::TestObject, test::TestObjectId, IdError};

  #[test]
  fn basic() {
    let mut test_store: ObjectStore<TestObject, TestObjectId> = ObjectStore::new();
    let t1 = test_store.insert_new(None, |id| Ok(TestObject::new(id, 100))).unwrap();
    let t2 = test_store.insert_new(None, |id| Ok(TestObject::new(id, 200))).unwrap();
    assert_ne!(t1, t2);

    // don't allow dupe
    let t1_dupe = TestObject::new(t1.clone(), 3);
    let dupe_result = test_store.register(None, t1_dupe);
    assert_eq!(dupe_result, Err(IdError::IdAlreadyExists(t1.clone())));

    // don't allow custom ids
    let testid_bad = TestObjectId::new(1000);
    let t_custom = test_store.insert_new(None, |_id| Ok(TestObject::new(testid_bad.clone(), 10)));
    assert_eq!(t_custom, Err(IdError::IdNotReserved(testid_bad)));

    // check values
    assert_eq!(test_store.get(&t1).unwrap().val(), 100);
    assert_eq!(test_store.get(&TestObjectId::new(999)), None);

    // callback failure
    assert_eq!(test_store.insert_new(None, |_id| Err(IdError::CannotParse("hi".to_owned()))), Err(IdError::CannotParse("hi".to_owned())));
  }

  #[test]
  fn register() {
    let mut test_store: ObjectStore<TestObject, TestObjectId> = ObjectStore::new();
    let id1 = test_id!(TestObjectId);
    let id2 = test_id!(TestObjectId);
    test_store.register(None, TestObject::new(id1, 100)).unwrap();
    test_store.register(None, TestObject::new(id2, 100)).unwrap();
    assert_eq!(test_store.register(None, TestObject::new(id1, 100)), Err(IdError::IdAlreadyExists(id1)));
  }

  #[test]
  fn names() {
    let mut test_store: ObjectStore<TestObject, TestObjectId> = ObjectStore::new();
    let t1 = test_store.insert_new(Some("t1".to_owned()), |id| Ok(TestObject::new(id, 100))).unwrap();
    let _t2 = test_store.insert_new(Some("t2".to_owned()), |id| Ok(TestObject::new(id, 200))).unwrap();

    // don't allow register dupe name
    let t1_dupe = test_store.insert_new(Some("t1".to_owned()), |id| Ok(TestObject::new(id, 150)));
    assert_eq!(t1_dupe, Err(IdError::NameAlreadyExists("t1".to_owned())));

    // check values
    assert_eq!(test_store.id_from_name("t1").unwrap().val(), t1.val());
    assert_eq!(test_store.get_by_name("t1").unwrap().val(), 100);
    assert_eq!(test_store.get_by_name("BAD"), None);
  }

  #[test]
  fn get() {
    let mut test_store: ObjectStore<TestObject, TestObjectId> = ObjectStore::new();
    let t1 = test_store.insert_new(Some("t1".to_owned()), |id| Ok(TestObject::new(id, 100))).unwrap();
    let _t2 = test_store.insert_new(Some("t2".to_owned()), |id| Ok(TestObject::new(id, 200))).unwrap();

    test_store.get_mut(&t1).unwrap().set_val(5);
    assert_eq!(test_store.get(&t1).unwrap().val(), 5);
  }
}
