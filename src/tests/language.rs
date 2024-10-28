use humantime::format_duration;
use rustmix::{
    ai::SourceSize,
    io::directory,
    language::{llma::*, *},
    threading::Spinner,
    Result,
};
use serde::de;
use std::{io::Write, time};
use tokio::sync::mpsc::unbounded_channel;

pub async fn test_llma() -> Result<()> {
    println!(
        "If this is the first time to run it, it will download the model and tokenizer files."
    );
    println!("After the model is downloaded, It can take a few seconds/minutes to initialize it.");
    println!("So have patience and wait for the model initialized message");

    let spinner = Spinner::new();
    spinner.set_message("Initializing model...");

    let llma = Llma::new(SourceSize::Small).await?;
    spinner.finish_with_message("Model initialized")?;

    loop {
        match llma.prompt("\nYou: ") {
            Ok(mut stream) => {
                match stream.next().await {
                    Some(text) => {
                        print!("Bot: {}", text);
                        std::io::stdout().flush()?
                    }
                    _ => break,
                }

                while let Some(text) = stream.next().await {
                    print!("{}", text);
                    std::io::stdout().flush()?
                }
            }
            Err(_) => break,
        }
    }

    Ok(())
}
