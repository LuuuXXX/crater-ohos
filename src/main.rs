use crater_ohos::prelude::*;

#[tokio::main]
async fn main() -> Fallible<()> {
    // Initialize logger
    env_logger::init();

    // Run CLI
    crater_ohos::cli::run().await
}
