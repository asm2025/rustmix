use anyhow::Result;

#[cfg(feature = "whisper")]
use rustmix::ai::whisper::*;

#[cfg(feature = "whisper")]
pub async fn test_whisper() -> Result<()> {
    let whisper = Whisper::new();
    let file_name = "test/captcha/fb1.mp3";
    let result = whisper.transcribe_file(&file_name).await?;
    println!("Whisper transcription: {}", result);
    Ok(())
}
