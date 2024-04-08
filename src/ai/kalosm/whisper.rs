use anyhow::Result;
use futures_util::stream::StreamExt;
use kalosm::{
    audio::{
        rodio::{buffer::SamplesBuffer, Decoder, Source},
        *,
    },
    language::*,
};
use std::{
    fs::File,
    io::{BufReader, Write},
};

pub struct Whisper {
    model: kalosm::audio::Whisper,
}

impl Whisper {
    pub fn new() -> Self {
        let model = WhisperBuilder::default()
            .with_source(WhisperSource::Tiny)
            .with_language(Some(WhisperLanguage::English))
            .build()
            .unwrap();
        Whisper { model }
    }

    pub async fn transcribe_file(&self, file_name: &str) -> Result<String> {
        let file = File::open(file_name)?;
        let source = Decoder::new(BufReader::new(file))?;
        let channels = source.channels();
        let sample_rate = source.sample_rate();
        let samples: Vec<f32> = source.convert_samples().collect();
        let buffer = SamplesBuffer::new(channels, sample_rate, samples);
        let transcription = self.model.transcribe(buffer)?.all_text().await;
        Ok(transcription)
    }
}
