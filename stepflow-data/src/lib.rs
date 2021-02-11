//! Data layer for [StepFlow](https://stepflow.dev)
//!
//! [`StateData`] is the main struct used to store data.
//! # Examples
//! ```
//! # use stepflow_data::{StateData, value::EmailValue, var::{VarId, EmailVar}};
//! // create the var/value combination
//! let email_var = EmailVar::new(VarId::new(0));
//! let email_val = EmailValue::try_new("test@stepflow.dev").unwrap();
//!
//! // insert it in a StateData
//! let mut statedata = StateData::new();
//! statedata.insert(&email_var.boxed(), email_val.boxed());
//! ```

mod statedata;
pub use statedata::StateData;

mod statedata_filtered;
pub use statedata_filtered::StateDataFiltered;

mod error;
pub use error::{InvalidValue, InvalidVars};

pub mod var;

#[cfg(test)]
use var::test_var_val;

mod base_value;
pub use base_value::{BaseValue};

pub mod value;
