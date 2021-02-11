use std::borrow::{Borrow, Cow};
use std::sync::{RwLock, RwLockWriteGuard};
use stepflow_base::{ObjectStore, IdError, ObjectStoreFiltered};
use stepflow_data::{StateDataFiltered, var::{Var, VarId}};
use stepflow_step::Step;
use stepflow_action::{ActionResult, Action, ActionId};
use super::{Error};


/// A wrapper to make it easier to access this ObjectStore that uses interior mutability.
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

  pub fn insert_new<CB>(&self, cb: CB) -> Result<ActionId, Error>
      where CB: FnOnce(ActionId) -> Result<Box<dyn Action + Sync + Send>, IdError<ActionId>>
  {
    self.action_store_write()
      .and_then(|mut store| {
        store.insert_new(cb).map_err(|e| Error::ActionId(e))
      })
  }

  pub fn insert_new_named<CB, STR>(&self, name: STR, cb: CB) -> Result<ActionId, Error>
      where CB: FnOnce(ActionId) -> Result<Box<dyn Action + Sync + Send>, IdError<ActionId>>,
            STR: Into<Cow<'static, str>>
  {
    self.action_store_write()
      .and_then(|mut store| {
        store.insert_new_named(name, cb).map_err(|e| Error::ActionId(e))
      })
  }

  pub fn register(&self, object: Box<dyn Action + Sync + Send>) -> Result<ActionId, Error> {
    self.action_store_write()
      .and_then(|mut store| {
        store.register(object).map_err(|e| Error::ActionId(e))
      })
  }

  pub fn register_named<STR>(&self, name: String, object: Box<dyn Action + Sync + Send>) -> Result<ActionId, Error>
      where STR: Into<Cow<'static, str>>
  {
    self.action_store_write()
      .and_then(|mut store| {
        store.register_named(name, object).map_err(|e| Error::ActionId(e))
      })
  }


  pub fn id_from_name<STR>(&self, name: STR) -> Result<ActionId, Error>
      where STR: Into<Cow<'static, str>>
  {
    let name: Cow<'static, str> = name.into();
    self.action_store_write()
      .and_then(|store| {
        store.id_from_name(name.borrow())
          .map(|id| id.clone())
          .ok_or_else(|| Error::ActionId(IdError::NoSuchName(name.into_owned())))
      })
  }

  pub fn name_from_id(&self, action_id: &ActionId) -> Result<String, Error> {
    self.action_store_write()
      .and_then(|store| {
        store.name_from_id(action_id)
          .map(|name| name.to_owned())
          .ok_or_else(|| Error::ActionId(IdError::IdMissing(action_id.clone())))
      })
  }

  pub fn start_action(&self, id: &ActionId, step: &Step, step_name: Option<&str>, step_data: &StateDataFiltered, vars: &ObjectStoreFiltered<Box<dyn Var + Send + Sync>, VarId>)
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
