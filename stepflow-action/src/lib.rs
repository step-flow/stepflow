//! Actions for [StepFlow](https://stepflow.dev)
//!
//! Provides the [`Action`] which fulfill the outputs of a [`Step`](stepflow_step::Step).
//!
//! Pre-built Actions include
//! - [`HtmlFormAction`]
//! - [`CallbackStepAction`]
//! - [`SetDataAction`]
//! - [`UrlStepAction`]

mod error;
pub use error::ActionError;

mod action;
pub use action::{ Action, ActionId, ActionResult, UrlStepAction, HtmlFormAction, HtmlFormConfig, SetDataAction, CallbackStepAction };

pub use http::Uri;