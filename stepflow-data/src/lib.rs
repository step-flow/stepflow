mod error;
pub use error::{InvalidValue, InvalidVars};

mod statedata;
pub use statedata::StateData;

mod statedata_filtered;
pub use statedata_filtered::StateDataFiltered;

mod var;
pub use var::{ Var, VarId, UriVar, StringVar, TrueVar, EmailVar, BoolVar };

#[cfg(test)]
use var::test_var_val;

mod base_value;
pub use base_value::{BaseValue};

mod value;
pub use value::{ Value, UriValue, StringValue, TrueValue, EmailValue, BoolValue };

mod value_valid;
pub use value_valid::ValidVal;
