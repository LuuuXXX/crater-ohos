#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

#[macro_use]
mod macros;

pub mod actions;
pub mod api;
pub mod cli;
pub mod config;
pub mod crates;
pub mod db;
pub mod dirs;
pub mod experiments;
pub mod platforms;
pub mod prelude;
pub mod report;
pub mod results;
pub mod runner;
pub mod server;
pub mod toolchain;
pub mod utils;

pub static USER_AGENT: &str = "crater-ohos";
