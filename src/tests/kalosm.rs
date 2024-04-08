use anyhow::Result;
use kalosm::language::TextStream;

use ::ai::kalosm::*;

use super::*;

pub async fn test_whisper() -> Result<()> {
    let whisper = Whisper::new();
    let file_name = "test/captcha/fb1.mp3";
    let result = whisper.transcribe_file(&file_name).await?;
    println!("Whisper transcription: {}", result);
    Ok(())
}

pub async fn test_phi() -> Result<()> {
    const LEN_MAX: usize = 300;

    let phi = Phi::new();

    loop {
        let prompt = stdin_input("Enter the prompt [ENTER to exit]: ");

        if prompt.is_empty() {
            break;
        }

        phi.generate_text(&prompt, LEN_MAX).await?.to_std_out();
    }

    Ok(())
}
