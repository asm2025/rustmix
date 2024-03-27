use anyhow::Result;

mod tests;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    //tests::io::test_path()?;
    //tests::io::test_directory()?;
    //tests::io::test_file()?;

    tests::web::test_url()?;
    //tests::web::test_reqwest().await?;

    //tests::mail::test_secmail().await?;
    //tests::mail::test_emailfake().await?;
    //tests::mail::test_tempmail().await?;

    //tests::threading::test_producer_consumer().await?;
    //tests::threading::test_consumer().await?;
    //tests::threading::test_injector_worker().await?;
    //tests::threading::test_parallel().await?;

    //tests::python::test_python();

    //tests::whisper::test_whisper().await?;

    // Cannot test building from config and code to test building loogers at the same time
    // Once logging is initialized, it cannot be reinitialized
    //tests::slog::test_slog()?;
    //tests::log4rs::test_log4rs(true)?;
    //tests::log4rs::test_log4rs(false)?;

    Ok(())
}
