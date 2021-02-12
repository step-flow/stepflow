//! Actions for [StepFlow](https://stepflow.dev)
//!
//! Provides the [`Action`] which fulfill the outputs of a [`Step`](stepflow_step::Step).
//!
//! Pre-built Actions include
//! - [`HtmlFormAction`]
//! - [`CallbackAction`]
//! - [`SetDataAction`]
//! - [`UrlAction`]

mod error;
pub use error::ActionError;

mod action;
pub use action::{ Action, ActionId, ActionResult, UrlAction, HtmlFormAction, HtmlFormConfig, SetDataAction, CallbackAction };

pub use http::Uri;