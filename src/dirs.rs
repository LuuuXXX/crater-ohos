use lazy_static::lazy_static;
use std::path::PathBuf;

lazy_static! {
    pub static ref WORK_DIR: PathBuf = {
        std::env::var_os("CRATER_WORK_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("work"))
    };
}
