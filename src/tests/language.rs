use humantime::format_duration;
use rustmix::{
    io::directory,
    language::{Bot, TextStream},
    threading::Spinner,
    Result,
};
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
    let bot = Bot::quick().await?;
    spinner.finish_with_message("Model initialized")?;
    println!();

    loop {
        match bot.prompt("You: ") {
            Ok(mut stream) => {
                stream.to_std_out().await?;
            }
            Err(_) => break,
        }
    }

    Ok(())
}
