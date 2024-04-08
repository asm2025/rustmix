use anyhow::Result;
use futures_util::stream::StreamExt;
use kalosm::language::*;
use std::{
    fs::File,
    io::{BufReader, Write},
};

pub struct Phi {
    model: kalosm::language::Phi,
}

impl Phi {
    pub fn new() -> Self {
        let model = kalosm::language::Phi::v2();
        Phi { model }
    }

    pub async fn generate_text(
        &self,
        prompt: &str,
        max_length: usize,
    ) -> Result<WordStream<ChannelTextStream<String>, String>> {
        if prompt.is_empty() || max_length == 0 {
            return Ok("".to_string());
        }

        let stream = self
            .model
            .stream_text(prompt)
            .with_max_length(max_length)
            .await?;
        Ok(stream.words())
    }
}
