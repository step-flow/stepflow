use crate::{generate_id_type, IdError, ObjectStoreContent};

generate_id_type!(TestObjectId);

#[derive(Debug, PartialEq)]
pub struct TestObject {
  id: TestObjectId,
  val: usize,
}

impl TestObject {
  pub fn new(id: TestObjectId, val: usize) -> Self {
    Self { id, val, }
  }

  pub fn val(&self) -> usize { self.val }

  pub fn set_val(&mut self, val: usize) {
    self.val = val;
  }
}

impl ObjectStoreContent for TestObject {
  type IdType = TestObjectId;

  fn new_id(id_val: u32) -> Self::IdType {
    TestObjectId::new(id_val)
  }

  fn id(&self) -> &Self::IdType {
    &self.id
  }
}