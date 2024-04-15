use anyhow::Result;
use image::ImageFormat;
use rustmix::{
    io::{
        directory,
        path::{AsPath, PathEx},
    },
    string::StringEx,
    vision::Image,
};
use std::path::MAIN_SEPARATOR;

use super::*;

pub async fn test_image() -> Result<()> {
    let curdir = (directory::current().as_str(), "out", "images")
        .as_path()
        .suffix(MAIN_SEPARATOR);
    let image = Image::new().await?;

    loop {
        let prompt = stdin_input("Enter a prompt to generate images: ");

        if prompt.is_empty() {
            break;
        }

        println!("Generating images...");
        directory::ensure(&curdir)?;

        if let Ok(images) = image.generate(&prompt).await {
            for (i, img) in images.iter().enumerate() {
                let filename = format!("{}IMG{:02}.png", curdir, i + 1);
                img.save_with_format(&filename, ImageFormat::Png)?;
            }
        } else {
            println!("Failed to generate images");
        }
    }

    Ok(())
}
