use std::error::Error;

mod tests;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();

    //tests::io::test_path()?;
    //tests::io::test_directory()?;
    //tests::io::test_file()?;

    //tests::web::test_url()?;
    //tests::web::test_reqwest().await?;
    tests::web::test_tmp_mail().await?;

    //tests::threading::test_producer_consumer().await?;
    //tests::threading::test_consumer().await?;
    //tests::threading::test_injector_worker().await?;
    //tests::threading::test_parallel().await?;

    //tests::python::test_python();

    Ok(())
}
