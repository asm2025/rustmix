use anyhow::Result;
use std::io::Write;

use rustmix::{ai::Whisper, io::path::AsPath};

pub async fn test_whisper() -> Result<()> {
    let whisper = Whisper::new();
    let file_name = ("test", "audio", "captcha", "fb1.mp3").as_path();
    println!("Transcribing file: {}", file_name);
    let result = whisper.transcribe_file(&file_name).await?;
    println!("Whisper transcription [text]: {}", result);

    let file_name = ("test", "audio", "listen.mp3").as_path();
    println!("Transcribing file: {}", file_name);
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    tokio::spawn(async move {
        print!("Whisper transcription [stream]: ");
        std::io::stdout().flush().unwrap();

        while let Some(result) = rx.recv().await {
            print!("{}", result);
            std::io::stdout().flush().unwrap();
        }

        println!()
    });

    whisper.transcribe_stream(&file_name, tx).await
}
