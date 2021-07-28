// #![feature(impl_trait_in_bindings)]
#![feature(maybe_uninit_slice)]
#![feature(duration_zero)]

mod core;
mod error;
mod packet;

pub use crate::core::Pinger;
pub use crate::error::PingError;
