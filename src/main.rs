use std::error::Error;

mod tests;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();

    // tests::io::test_path_func()?;
    // tests::io::test_directory_func()?;
    // tests::io::test_file_func()?;

    // tests::web::test_url_func()?;
    // tests::web::test_reqwest_func().await?;

    tests::threading::test_prodcon(4).await?;

    Ok(())
}
