use humantime::format_duration;
use rustmix::{io::directory, sound::*, threading::Spinner, Result};
use std::{io::Write, time};
use tokio::sync::mpsc::unbounded_channel;

pub async fn test_sound() -> Result<()> {
    println!(
        "If this is the first time to run it, it will download the model and tokenizer files."
    );
    println!("After the model is downloaded, It can take a few seconds/minutes to initialize it.");
    println!("So have patience and wait for the model initialized message");

    let spinner = Spinner::new();
    spinner.set_message("Initializing audio model...");
    let sound = Audio::with_source(WhisperSource::DistilLargeV3).await?;
    spinner.finish_with_message("Audio model initialized")?;
    println!();

    let curdir = directory::current().join("files/audio");

    let file_name = curdir.join("awz1.mp3");
    let base_name = file_name
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    spinner.reset()?;
    spinner.set_message(format!("Transcribing file [text]: {}", &base_name));
    let snd = sound.clone();
    let timer = time::Instant::now();
    let result = spinner.run(move || snd.transcribe_file(&file_name).unwrap())?;
    spinner.finish_with_message(format!(
        "Sound transcription [{}]: '{}'",
        &base_name, result
    ))?;
    println!("Time elapsed: {}", format_duration(timer.elapsed()));

    let file_name = curdir.join("fb1.mp3");
    let base_name = file_name
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    spinner.reset()?;
    spinner.set_message(format!("Transcribing file [text]: {}", &base_name));
    let snd = sound.clone();
    let timer = time::Instant::now();
    let result = spinner.run(move || snd.transcribe_file(&file_name).unwrap())?;
    spinner.finish_with_message(format!(
        "Sound transcription [{}]: '{}'",
        &base_name, result
    ))?;
    println!("Time elapsed: {}", format_duration(timer.elapsed()));

    let file_name = curdir.join("fbn.mp3");
    let base_name = file_name
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    spinner.reset()?;
    spinner.set_message(format!("Transcribing file [text]: {}", &base_name));
    let snd = sound.clone();
    let timer = time::Instant::now();
    let result = spinner.run(move || snd.transcribe_file(&file_name).unwrap())?;
    spinner.finish_with_message(format!(
        "Sound transcription [{}]: '{}'",
        &base_name, result
    ))?;
    println!("Time elapsed: {}", format_duration(timer.elapsed()));

    let file_name = curdir.join("pinless.wav");
    let base_name = file_name
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    spinner.reset()?;
    spinner.set_message(format!("Transcribing file [text]: {}", &base_name));
    let snd = sound.clone();
    let timer = time::Instant::now();
    let result = spinner.run(move || snd.transcribe_file(&file_name).unwrap())?;
    spinner.finish_with_message(format!(
        "Sound transcription [{}]: '{}'",
        &base_name, result
    ))?;
    println!("Time elapsed: {}", format_duration(timer.elapsed()));

    let file_name = curdir.join("listen1.mp3");
    let base_name = file_name
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    println!("Transcribing file [callback]: {}", &base_name);
    let snd = sound.clone();
    let timer = time::Instant::now();
    print!("Sound transcription [{}]: '", &base_name);
    std::io::stdout().flush().unwrap();
    snd.transcribe_file_callback(&file_name, move |e| {
        print!("{}", e);
        std::io::stdout().flush().unwrap();
    })
    .unwrap();
    println!("'");
    println!("Time elapsed: {}", format_duration(timer.elapsed()));

    let file_name = curdir.join("listen2.mp3");
    let base_name = file_name
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    println!("Transcribing file [stream]: {}", &base_name);
    let (tx, mut rx) = unbounded_channel::<Segment>();
    let handle = tokio::spawn(async move {
        print!("Sound transcription [{}]: '", &base_name);
        std::io::stdout().flush().unwrap();

        while let Some(result) = rx.recv().await {
            print!("{}", result.text());
            std::io::stdout().flush().unwrap();
        }
    });
    let timer = time::Instant::now();
    sound.transcribe_stream(&file_name, tx)?;
    handle.await?;
    println!("'");
    println!("Time elapsed: {}", format_duration(timer.elapsed()));

    Ok(())
}
