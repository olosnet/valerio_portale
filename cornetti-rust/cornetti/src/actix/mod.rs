#![doc = include_str!("../../../spec/actix.md")]

pub mod adapters;
pub mod models;
pub mod pagination;

#[cfg(feature = "actix-filemanager")]
pub mod filemanager;

#[cfg(feature = "actix-auth")]
pub mod auth;

#[cfg(feature = "actix-auth-apikey")]
pub mod auth_apikey;

pub mod helpers;
