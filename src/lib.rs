#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

pub mod config;
pub mod db;
pub mod dirs;
pub mod prelude;
pub mod utils;

pub static USER_AGENT: &str = "crater-ohos";
