use std::sync::{RwLock, RwLockWriteGuard};
use stepflow_base::{ObjectStore, IdError, ObjectStoreFiltered};
use stepflow_data::{Var, VarId, StateDataFiltered};
use stepflow_step::Step;
use stepflow_action::{ActionResult, Action, ActionId};
use super::{Error};


// A wrapper to make it easier to access this ObjectStore that uses interior mutability.
// Someday it would be nice to get rid of it and use the normal ObjectStore instead
#[derive(Debug)]
pub struct ActionObjectStore {
  object_store: RwLock<ObjectStore<Box<dyn Action + Sync + Send>, ActionId>>,
}

impl ActionObjectStore {
  pub fn with_capacity(capacity: usize) -> ActionObjectStore {
    ActionObjectStore {
      object_store: RwLock::new(ObjectStore::with_capacity(capacity)),
    }
  }

  fn action_store_write(&self) 
      -> Result<RwLockWriteGuard<ObjectStore<Box<dyn Action + Sync + Send>, ActionId>>, Error>
  {
    let store = self
      .object_store
      .try_write()
      .map_err(|_e| Error::Other)?;
    Ok(store)
  }

  pub fn reserve_id(&self) -> Result<ActionId, Error> {
    self.action_store_write()
     .map(|mut store| store.reserve_id())
  }

  pub fn insert_new<CB>(&self, name: Option<String>, cb: CB) -> Result<ActionId, Error>
      where CB: FnOnce(ActionId) -> Result<Box<dyn Action + Sync + Send>, IdError<ActionId>>
  {
    self.action_store_write()
      .and_then(|mut store| {
        store.insert_new(name, cb).map_err(|e| Error::ActionId(e))
      })
  }

  pub fn register(&self, name: Option<String>, object: Box<dyn Action + Sync + Send>) -> Result<ActionId, Error> {
    self.action_store_write()
      .and_then(|mut store| {
        store.register(name, object).map_err(|e| Error::ActionId(e))
      })
  }

  pub fn id_from_name(&self, name: &str) -> Result<ActionId, Error> {
    self.action_store_write()
      .and_then(|store| {
        store.id_from_name(name)
          .map(|id| id.clone())
          .ok_or_else(|| Error::ActionId(IdError::NoSuchName(name.to_owned())))
      })
  }

  pub fn name_from_id(&self, action_id: &ActionId) -> Result<String, Error> {
    self.action_store_write()
      .and_then(|store| {
        store.name_from_id(action_id)
          .map(|name| name.clone())
          .ok_or_else(|| Error::ActionId(IdError::IdMissing(action_id.clone())))
      })
  }

  pub fn start_action(&self, id: &ActionId, step: &Step, step_name: Option<&String>, step_data: &StateDataFiltered, vars: &ObjectStoreFiltered<Box<dyn Var + Send + Sync>, VarId>)
      -> Result<ActionResult, Error>
  {
    self.action_store_write()
      .and_then(|mut store| {
        let action = store.get_mut(id).ok_or_else(|| Error::ActionId(IdError::IdMissing(id.clone())))?;
        action.start(&step, step_name, &step_data, &vars)
          .map_err(|e| e.into())
      })
  }
}
