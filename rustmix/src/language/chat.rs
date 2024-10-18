pub use kalosm::language::*;
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use crate::{error::*, Result};

#[derive(Clone)]
pub struct Bot {
    model: Arc<Mutex<Chat>>,
}

impl Bot {
    pub async fn quick() -> Result<Self> {
        let model = Llama::builder()
            .with_source(LlamaSource::llama_3_1_8b_chat())
            .build()
            .await?;
        let chat = Chat::builder(model).build();
        Ok(Bot {
            model: Arc::new(Mutex::new(chat)),
        })
    }

    pub async fn phi() -> Result<Self> {
        let model = Llama::builder()
            .with_source(LlamaSource::phi_3_5_mini_4k_instruct())
            .build()
            .await?;
        let chat = Chat::builder(model).build();
        Ok(Bot {
            model: Arc::new(Mutex::new(chat)),
        })
    }

    pub async fn new() -> Result<Self> {
        let model = Llama::builder()
            .with_source(LlamaSource::llama_8b())
            .build()
            .await?;
        let chat = Chat::builder(model).build();
        Ok(Bot {
            model: Arc::new(Mutex::new(chat)),
        })
    }

    pub async fn with_source(source: LlamaSource) -> Result<Self> {
        let model = Llama::builder().with_source(source).build().await?;
        let chat = Chat::builder(model).build();
        Ok(Bot {
            model: Arc::new(Mutex::new(chat)),
        })
    }

    pub fn prompt<T: AsRef<str>>(&self, prompt: T) -> Result<ChannelTextStream> {
        let prompt = prompt.as_ref();
        let prompt = if prompt.is_empty() { "\n>" } else { prompt };
        match prompt_input(prompt) {
            Ok(prompt) => {
                let mut model = self.model.lock().unwrap();
                Ok(model.add_message(prompt))
            }
            Err(e) => Err(e.into()),
        }
    }

    pub fn send<T: AsRef<str>>(&self, prompt: T) -> Result<ChannelTextStream> {
        let prompt = prompt.as_ref();

        if prompt.is_empty() {
            return Err(NoInputError.into());
        }

        let mut model = self.model.lock().unwrap();
        Ok(model.add_message(prompt))
    }

    pub fn history(&self) -> Vec<ChatHistoryItem> {
        self.model.lock().unwrap().history()
    }

    pub async fn save<T: AsRef<Path>>(&self, path: T) -> Result<()> {
        let mut model = self.model.lock().unwrap();
        model.save_session(path).await.map_err(Into::into)
    }
}
