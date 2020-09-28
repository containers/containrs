//! This is the main library interface for this project
#![deny(missing_docs)]

mod config;
mod cri_service;
mod criapi;
mod image_service;
mod network;
mod oci_spec;
mod runtime_service;
mod sandbox;
mod server;
mod storage;
mod unix_stream;

pub use config::Config;
pub use server::Server;

#[macro_use]
extern crate bitflags;
