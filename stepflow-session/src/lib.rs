//! Primary layer for managing a flow for [StepFlow](https://stepflow.dev)
//!
//! [`Session`] is the primary interface for creating and managing a flow.

mod session;
pub use session::{ Session, SessionId, AdvanceBlockedOn, ActionObjectStore };

mod errors;
pub use errors::Error;

mod dfs;

#[cfg(test)]
mod test;