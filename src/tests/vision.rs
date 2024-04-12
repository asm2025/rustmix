use anyhow::Result;
use image::ImageFormat;
use rustmix::{
    ai::Image,
    io::{
        directory,
        path::{self, AsPath, PathExt},
    },
    string::StringEx,
};
use std::io::Write;

use super::*;

pub async fn test_image() -> Result<()> {
    let image = Image::new().await?;

    loop {
        let prompt = stdin_input("Enter a prompt to generate images: ");

        if prompt.is_empty() {
            break;
        }

        println!("Generating images");

        if let Ok(images) = image.generate(&prompt).await {
            for (i, img) in images.iter().enumerate() {}
        } else {
            println!("Failed to generate images");
        }
    }

    Ok(())
}
