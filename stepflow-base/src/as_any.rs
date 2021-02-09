use std::any::Any;

/// Get the `Any` trait easily from objects that support it.
pub trait AsAny {
  fn as_any(&self) -> &dyn Any;  
}

impl<T: Any> AsAny for T {
  fn as_any(&self) -> &dyn Any {
      self
  }
}

