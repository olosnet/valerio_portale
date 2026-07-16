#![doc = include_str!("../../README.md")]

pub mod core;

#[cfg(feature = "filemanager")]
pub mod filemanager;

#[cfg(feature = "mongo")]
pub mod mongo;

#[cfg(feature = "redisdb")]
pub mod redis;

#[cfg(feature = "actix")]
pub mod actix;

#[cfg(feature = "auth")]
pub mod auth;

#[cfg(feature = "auth-apikey")]
pub mod auth_apikey;

#[cfg(feature = "templates")]
pub mod templates;

#[cfg(feature = "mail")]
pub mod mail;

#[cfg(feature = "sqlxdb")]
pub mod sqlx;

#[cfg(feature = "grpc")]
pub mod grpc;

#[cfg(feature = "otp")]
pub mod otp;
