use anyhow::Result;
use rwhisper::Segment;
use std::io::Write;

use rustmix::{ai::Whisper, io::path::AsPath};

pub async fn test_whisper() -> Result<()> {
    let whisper = Whisper::new();
    let file_name = ("test", "audio", "captcha", "fb1.mp3").as_path();
    println!("Transcribing file [text]: {}", file_name);
    let result = whisper.transcribe_file(&file_name).await?;
    println!("Whisper transcription: {}", result);

    let file_name = ("test", "audio", "captcha", "fb2.mp3").as_path();
    println!("Transcribing file [text]: {}", file_name);
    let result = whisper.transcribe_file(&file_name).await?;
    println!("Whisper transcription: {}", result);

    let file_name = ("test", "audio", "listen1.mp3").as_path();
    println!("Transcribing file [file_callback]: {}", file_name);
    print!("Whisper transcription: ");
    std::io::stdout().flush().unwrap();
    whisper
        .transcribe_file_callback(&file_name, |result| {
            print!("{}", result);
            std::io::stdout().flush().unwrap();
        })
        .await?;
    println!();

    let file_name = ("test", "audio", "listen2.mp3").as_path();
    println!("Transcribing file [stream]: {}", file_name);
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Segment>();
    tokio::spawn(async move {
        print!("Whisper transcription: ");
        std::io::stdout().flush().unwrap();

        while let Some(result) = rx.recv().await {
            print!("{}", result.text());
            std::io::stdout().flush().unwrap();
        }

        println!()
    });

    whisper.transcribe_stream(&file_name, tx).await
}
