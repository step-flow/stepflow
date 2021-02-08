mod errors;
pub use errors::IdError;

pub mod id;

mod object_store;
pub use object_store::{ ObjectStore, ObjectStoreContent };

mod object_store_filtered;
pub use object_store_filtered::ObjectStoreFiltered;

// NOTE: we don't do a broad use of as_any so we can be specific which objects should support the trait.
// i.e. if Box<T> gets it via blanket implementation, then we'll have to remember to do boxed.as_ref().as_any() as opposed to boxed.as_any()
pub mod as_any;

#[cfg(test)]
mod test;