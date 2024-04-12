use anyhow::Result;
use rustmix::{
    ai::sound::{Audio, Segment},
    io::{
        directory,
        path::{AsPath, PathExt},
    },
};
use std::io::Write;

pub async fn test_sound() -> Result<()> {
    let sound = Audio::quick().await?;
    let curdir = (directory::current().as_str(), "..", "files", "audio").as_full_path();
    let file_name = (curdir.as_str(), "captcha", "fb1.mp3").as_path();
    println!("Transcribing file [text]: {}", file_name);
    let result = sound.transcribe_file(&file_name).await?;
    println!("Sound transcription: {}", result);

    let file_name = (curdir.as_str(), "captcha", "fb2.mp3").as_path();
    println!("Transcribing file [text]: {}", file_name);
    let result = sound.transcribe_file(&file_name).await?;
    println!("Sound transcription: {}", result);

    let file_name = (curdir.as_str(), "listen1.mp3").as_path();
    println!("Transcribing file [file_callback]: {}", file_name);
    print!("Sound transcription: ");
    std::io::stdout().flush().unwrap();
    sound
        .transcribe_file_callback(&file_name, |result| {
            print!("{}", result);
            std::io::stdout().flush().unwrap();
        })
        .await?;
    println!();

    let file_name = (curdir.as_str(), "listen2.mp3").as_path();
    println!("Transcribing file [stream]: {}", file_name);
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Segment>();
    tokio::spawn(async move {
        print!("Sound transcription: ");
        std::io::stdout().flush().unwrap();

        while let Some(result) = rx.recv().await {
            print!("{}", result.text());
            std::io::stdout().flush().unwrap();
        }

        println!()
    });

    sound.transcribe_stream(&file_name, tx).await
}
