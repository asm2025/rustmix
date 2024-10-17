use humantime::format_duration;
use rustmix::{io::directory, language::*, threading::Spinner, Result};
use std::{io::Write, time};
use tokio::sync::mpsc::unbounded_channel;

pub async fn test_chat() -> Result<()> {
    println!(
        "If this is the first time to run it, it will download the model and tokenizer files."
    );
    println!("After the model is downloaded, It can take a few seconds/minutes to initialize it.");
    println!("So have patience and wait for the model initialized message");

    let spinner = Spinner::new();
    spinner.set_message("Initializing LLMA model...");
    let chat = Chat::quick().await?;
    spinner.finish_with_message("LLMA model initialized")?;
    println!();

    let prompt = "The following is a 300 word essay about Paris:";
    println!("You: {}", prompt);
    print!("Bot: ");
    chat.callback_async(prompt, Some(1000), |s| {
        print!("{}", s);
        std::io::stdout().flush().unwrap();
    })
    .await?;
    println!();

    loop {
        print!("You: ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            break;
        }

        print!("Bot: ");
        std::io::stdout().flush()?;

        let timer = time::Instant::now();
        chat.callback_async(input, None, |s| {
            print!("{}", s);
            std::io::stdout().flush().unwrap();
        })
        .await?;
        println!();
        println!("Time elapsed: {}", format_duration(timer.elapsed()));
        println!();
    }

    Ok(())
}
