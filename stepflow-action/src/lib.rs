//! Actions for [StepFlow](https://stepflow.dev)
//!
//! Provides the [`Action`] which fulfill the outputs of a [`Step`](stepflow_step::Step).
//!
//! Pre-built Actions include
//! - [`HtmlFormAction`]
//! - [`CallbackAction`]
//! - [`SetDataAction`]

mod error;
pub use error::ActionError;

mod string_template;
pub use string_template::{render_template, EscapedString, HtmlEscapedString, UriEscapedString};

mod action;
pub use action::{ Action, ActionId, ActionResult, StringTemplateAction, HtmlFormAction, HtmlFormConfig, SetDataAction, CallbackAction };
