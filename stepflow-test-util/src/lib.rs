pub fn test_id_val() -> u32 {
  use std::sync::atomic::{AtomicU32, Ordering};
  static COUNT: AtomicU32 = AtomicU32::new(0);

  // add extra bits to make it easy to identiy test IDs
  (u16::MAX as u32) << 16 | COUNT.fetch_add(1, Ordering::SeqCst)
}

#[macro_export]
macro_rules! test_id {
  ($id_type:ident) => {
    $id_type::new(stepflow_test_util::test_id_val())
  }
}
