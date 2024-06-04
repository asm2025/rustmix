use image::ImageFormat;
use rustmix::{
    input,
    io::{
        directory,
        path::{self, AsPath, PathEx},
    },
    string::StringEx,
    threading::{Spinner, INTERVAL},
    vision::Image,
    Result,
};
use std::path::MAIN_SEPARATOR;

pub async fn test_image() -> Result<()> {
    let curdir = (directory::current().as_str(), "out", "images")
        .as_path()
        .suffix(MAIN_SEPARATOR);
    println!(
        "If this is the first time to run it, it will download the model and tokenizer files."
    );
    println!("After the model is downloaded, It can take a few seconds/minutes to initialize it.");
    println!("So have patience and wait for the model initialized message");

    let spinner = Spinner::new();
    spinner.set_message("Initializing image model...");
    let image = Image::new().await?;
    spinner.finish_with_message("Image model initialized")?;

    loop {
        let prompt = input::get("Enter a prompt to generate images: ")?;

        if prompt.is_empty() {
            break;
        }

        println!("Images will be saved to '{}'", curdir);

        if !directory::is_empty(&curdir) {
            if !input::confirm("Clearing the directory. Press any key to continue. [y] ")? {
                return Ok(());
            }

            path::del(&curdir)?;
        } else {
            directory::ensure(&curdir)?;
        }

        spinner.reset()?;
        spinner.set_steady_tick(INTERVAL);
        spinner.set_message("Generating images...");

        if let Ok(images) = image.generate(&prompt) {
            spinner.finish_with_message("Images generated")?;

            for (i, img) in images.iter().enumerate() {
                let filename = format!("{}IMG{:02}.png", curdir, i + 1);
                img.save_with_format(&filename, ImageFormat::Png)?;
            }

            println!("Images saved");
        } else {
            spinner.finish_with_message("Failed to generate images")?;
        }
    }

    Ok(())
}
