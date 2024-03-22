use anyhow::Result;
use kalosm::{
    audio::{
        rodio::{buffer::SamplesBuffer, Decoder, Source},
        *,
    },
    language::TextStream,
};
use std::{fs::File, io::BufReader};

pub struct Whisper {
    model: kalosm::audio::Whisper,
}

/*
This needs the following dependencies:
Debian:
    sudo apt-get install libasound2-dev
    sudo apt-get update
    sudo apt-get install -y build-essential libgflags-dev libsnappy-dev zlib1g-dev libbz2-dev liblz4-dev libzstd-dev
    sudo apt install clang
Fedora:
    sudo dnf install alsa-lib-devel

Then do:
    git clone https://github.com/facebook/rocksdb.git
    cd rocksdb
    make shared_lib
    sudo make install
*/
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
