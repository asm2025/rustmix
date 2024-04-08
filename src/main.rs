use anyhow::Result;
use tokio::{task, time::Duration};

mod tests;

#[tokio::main]
async fn main() -> Result<()> {
    // **** Make sure the feature is enabled in Cargo.toml before tsting ****
    dotenv::dotenv().ok();

    //tests::io::test_path()?;
    //tests::io::test_directory()?;
    //tests::io::test_file()?;

    //tests::web::test_url()?;
    //tests::web::test_reqwest().await?;
    //task::spawn_blocking(move || tests::web::test_blocking_reqwest().unwrap()).await?;

    //tests::mail::test_tempmail().await?;

    //tests::threading::test_producer_consumer(Duration::ZERO).await?;
    //tests::threading::test_producer_consumer(Duration::from_millis(150)).await?;
    //tests::threading::test_consumer(Duration::ZERO).await?;
    //tests::threading::test_consumer(Duration::from_millis(150)).await?;
    //tests::threading::test_injector_worker(Duration::ZERO).await?;
    //tests::threading::test_injector_worker(Duration::from_millis(150)).await?;
    //tests::threading::test_parallel(Duration::ZERO).await?;
    //tests::threading::test_parallel(Duration::from_millis(150)).await?;

    //tests::python::test_python();

    //tests::kalosm::test_whisper().await?;
    //tests::kalosm::test_phi().await?;

    /*
     Once logging is initialized, it cannot be reinitialized.
     Therefore, to test configuring log from a configuration file and code at the same time is not possible.
     It has to be done seperately.
    */
    //tests::slog::test_slog()?;
    //tests::log4rs::test_log4rs(true)?;
    //tests::log4rs::test_log4rs(false)?;

    //tests::app::test_app_info();

    Ok(())
}
