use humantime::format_duration;
use rustmix::{io::directory, language::*, threading::Spinner, Result};
use serde::de;
use std::{io::Write, time};
use tokio::sync::mpsc::unbounded_channel;

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

    loop {
        match llma.prompt(
            "\nYou: ",
            Some(|| {
                print!("Thinking...");
            }),
        ) {
            Ok(mut stream) => {
                match stream.next().await {
                    Some(text) => {
                        print!("\nBot: {}", text);
                        std::io::stdout().flush()?
                    }
                    _ => break,
                }

                while let Some(text) = stream.next().await {
                    print!("{}", text);
                    std::io::stdout().flush()?
                }

                println!()
            }
            Err(_) => break,
        }
    }

    Ok(())
}
