use humantime::format_duration;
use rustmix::{io::directory, language::*, threading::Spinner, Result};
use serde::de;
use std::{io::Write, time};
use tokio::sync::mpsc::unbounded_channel;

#[derive(Schema, Parse, Debug, Clone)]
enum Language {
    Rust,
    C,
    CPP,
    Java,
    CSharp,
    VisualBasic,
    JavaScript,
    VBScript,
    Python,
    Ruby,
}

pub async fn test_chat() -> Result<()> {
    println!(
        "If this is the first time to run it, it will download the model and tokenizer files."
    );
    println!("After the model is downloaded, It can take a few seconds/minutes to initialize it.");
    println!("So have patience and wait for the model initialized message");

    let spinner = Spinner::new();
    spinner.set_message("Initializing model...");
    let llma = Llma::quick().await?;
    spinner.finish_with_message("Model initialized")?;
    println!();

    loop {
        match llma.prompt("\nYou: ") {
            Ok(mut stream) => {
                stream.to_std_out().await?;
            }
            Err(_) => break,
        }
    }

    Ok(())
}
