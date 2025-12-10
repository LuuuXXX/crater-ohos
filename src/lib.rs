#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

#[macro_use]
mod macros;

pub mod config;
pub mod crates;
pub mod db;
pub mod dirs;
pub mod experiments;
pub mod prelude;
pub mod results;
pub mod toolchain;
pub mod utils;

pub static USER_AGENT: &str = "crater-ohos";
