pub fn test_id_val() -> u16 {
  use std::sync::atomic::{AtomicU16, Ordering};
  static COUNT: AtomicU16 = AtomicU16::new(0);

  // add extra bits to make it easy to identiy test IDs
  (u8::MAX as u16) << 8 | COUNT.fetch_add(1, Ordering::SeqCst)
}

#[macro_export]
macro_rules! test_id {
  ($id_type:ident) => {
    $id_type::new(stepflow_test_util::test_id_val())
  }
}
