use rustmix::{
    io::{
        directory,
        path::{AsPath, PathEx},
    },
    sound::*,
    threading::Spinner,
    Result,
};
use std::{cell::RefCell, io::Write};
use tokio::sync::mpsc::unbounded_channel;

pub async fn test_sound() -> Result<()> {
    println!(
        "If this is the first time to run it, it will download the model and tokenizer files."
    );
    println!("After the model is downloaded, It can take a few seconds/minutes to initialize it.");
    println!("So have patience and wait for the model initialized message");

    let spinner = Spinner::new();
    spinner.set_message("Initializing audio model...".to_string());
    let sound = Audio::with_source(WhisperSource::BaseEn).await?;
    spinner.finish_with_message("Audio model initialized".to_string())?;

    let curdir = (directory::current().as_str(), "files", "audio").as_path();
    let file_name = (curdir.as_str(), "captcha", "fb1.mp3").as_path();
    spinner.reset()?;
    spinner.set_message(format!("Transcribing file [text]: {}", &file_name));
    let snd = sound.clone();
    let result = spinner.run(move || snd.transcribe_file(&file_name).unwrap())?;
    spinner.finish_with_message(format!("Sound transcription: {}", result))?;

    let file_name = (curdir.as_str(), "captcha", "fb2.mp3").as_path();
    spinner.reset()?;
    spinner.set_message(format!("Transcribing file [text]: {}", &file_name));
    let snd = sound.clone();
    let result = spinner.run(move || snd.transcribe_file(&file_name).unwrap())?;
    spinner.finish_with_message(format!("Sound transcription: {}", result))?;

    let file_name = (curdir.as_str(), "listen1.mp3").as_path();
    spinner.reset()?;
    spinner.set_message(format!("Transcribing file [file_callback]: {}", &file_name));
    let snd = sound.clone();
    let result = spinner.run(move || {
        let r = RefCell::new(String::new());
        snd.transcribe_file_callback(&file_name, |text| {
            r.borrow_mut().push_str(&text);
        })
        .unwrap();
        r.into_inner()
    })?;
    spinner.finish_with_message(format!("Sound transcription: {}", result))?;

    let file_name = (curdir.as_str(), "listen2.mp3").as_path();
    println!("Transcribing file [stream]: {}", &file_name);
    let (tx, mut rx) = unbounded_channel::<Segment>();
    tokio::spawn(async move {
        print!("Sound transcription: ");
        std::io::stdout().flush().unwrap();

        while let Some(result) = rx.recv().await {
            print!("{}", result.text());
            std::io::stdout().flush().unwrap();
        }

        println!()
    });

    sound.transcribe_stream(&file_name, tx)
}
