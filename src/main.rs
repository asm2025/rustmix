#![allow(unused_imports)]
#![allow(dead_code)]
mod tests;

use rustmix::{error::*, set_debug, Result};
use tokio::{task, time::Duration};

#[tokio::main]
async fn main() -> Result<()> {
    /*
     **** Make sure the feature is enabled in Cargo.toml before tsting ****
     Once logging is initialized, it cannot be reinitialized.
     Therefore, to test configuring log from a configuration file and code at the same time is not possible.
     It has to be done seperately.
    */
    dotenv::dotenv().ok();

    // set_debug(true);
    // println!("{}", CanceledError.get_message());

    //tests::test_app_info();

    //tests::test_random();

    //tests::test_path()?;

    //tests::test_path()?;
    //tests::test_directory()?;
    //tests::test_file()?;

    //tests::test_url()?;
    //tests::test_reqwest().await?;
    //task::spawn_blocking(move || tests::test_blocking_reqwest().unwrap()).await?;

    //tests::test_slog()?;
    //tests::test_log4rs(true)?;
    //tests::test_log4rs(false)?;

    //tests::test_tempmail().await?;

    //tests::test_consumer(Duration::ZERO).await?;
    //tests::test_consumer(Duration::from_millis(150)).await?;
    //tests::test_producer_consumer(Duration::ZERO).await?;
    //tests::test_producer_consumer(Duration::from_millis(150)).await?;
    //tests::test_injector_worker(Duration::ZERO).await?;
    //tests::test_injector_worker(Duration::from_millis(150)).await?;

    //tests::test_sound().await?;
    tests::test_chat().await?;
    //tests::test_image().await?;

    //tests::test_expressvpn().await?;

    Ok(())
}
