#![doc = include_str!("../../../spec/redis.md")]

#[cfg(feature = "auth")]
pub mod auth;
pub mod confs;
pub mod errors;
pub mod services;
