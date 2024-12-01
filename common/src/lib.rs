//! Common definitions.
//!
//! List of available Cargo features:
#![doc = document_features::document_features!()]
#![deny(
    nonstandard_style,
    rust_2018_idioms,
    rustdoc::all,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code
)]
#![forbid(non_ascii_idents)]
#![warn(
    clippy::allow_attributes,
    clippy::allow_attributes_without_reason,
    clippy::pedantic,
    clippy::wildcard_enum_match_arm,
    deprecated_in_future,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    unused_crate_dependencies,
    unused_import_braces,
    unused_labels,
    unused_lifetimes,
    unused_qualifications,
    unused_results
)]

pub mod datetime;
pub mod handler;
mod kind;
pub mod money;
pub mod operations;
pub mod pagination;
mod percent;
pub mod unit;

pub use self::{
    datetime::{DateTime, DateTimeOf},
    handler::Handler,
    kind::FromParam,
    money::Money,
    percent::Percent,
};

#[doc(hidden)]
pub mod private {
    //! Private definitions used by macros.

    #[cfg(feature = "postgres")]
    pub use postgres_types;
    #[cfg(feature = "serde")]
    pub use serde;
    pub use strum;
}
