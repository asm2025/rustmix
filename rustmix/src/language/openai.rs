use async_openai::*;
use config::Config;
use futures::StreamExt;
use kalosm::language::prompt_input;
use reqwest::Client as ReqwestClient;
use std::{
    fmt,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs};

use crate::{ai::SourceSize, error::*, Result};

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OpenAiSource {
    gpt_3_5_turbo,
    #[default]
    gpt_4o_mini,
    gpt_4o,
    gpt_4,
    gpt_4_turbo,
    o1_mini,
}

impl fmt::Display for OpenAiSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OpenAiSource::gpt_3_5_turbo => write!(f, "gpt-3.5-turbo"),
            OpenAiSource::gpt_4o_mini => write!(f, "gpt-4o-mini"),
            OpenAiSource::gpt_4o => write!(f, "gpt-4o"),
            OpenAiSource::gpt_4 => write!(f, "gpt-4"),
            OpenAiSource::gpt_4_turbo => write!(f, "gpt-4-turbo"),
            OpenAiSource::o1_mini => write!(f, "o1-mini"),
        }
    }
}

#[derive(Clone)]
pub struct ChatGpt<C: Config> {
    client: Arc<Client<C>>,
    source: OpenAiSource,
    max_tokens: u32,
    sender: UnboundedSender<String>,
    receiver: Arc<Mutex<UnboundedReceiver<String>>>,
}

impl<C: Config> ChatGpt<C> {
    pub fn quick(config: C) -> Self {
        Self::with(
            config,
            Some(OpenAiSource::default()),
            ReqwestClient::new(),
            Default::default(),
        )
    }

    pub fn new(config: C, size: SourceSize) -> Self {
        let source = match size {
            SourceSize::Tiny => OpenAiSource::gpt_4o_mini,
            SourceSize::Small | SourceSize::Base => OpenAiSource::gpt_4o,
            SourceSize::Medium => OpenAiSource::gpt_4,
            SourceSize::Large => OpenAiSource::gpt_4_turbo,
        };
        Self::with(
            config,
            Some(source),
            ReqwestClient::new(),
            Default::default(),
        )
    }

    pub fn with_client(config: C, client: ReqwestClient, source: Option<OpenAiSource>) -> Self {
        Self::with(config, source, client, Default::default())
    }

    pub fn with(
        config: C,
        source: Option<OpenAiSource>,
        client: ReqwestClient,
        backoff: backoff::ExponentialBackoff,
    ) -> Self {
        let (sender, receiver) = unbounded_channel();
        Self {
            client: Arc::new(Client::build(client, config, backoff)),
            source: source.unwrap_or_default(),
            max_tokens: 1024u32,
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    pub fn subscribe(&self) -> Arc<Mutex<UnboundedReceiver<String>>> {
        Arc::clone(&self.receiver)
    }

    pub async fn prompt<T: AsRef<str>>(&self, prompt: T) -> Result<()> {
        let prompt = prompt.as_ref();
        let prompt = if prompt.is_empty() { "\n>" } else { prompt };
        let prompt = prompt_input(prompt)?;
        if prompt.is_empty() {
            return Err(NoInputError.into());
        }

        let messages = [ChatCompletionRequestUserMessageArgs::default()
            .content(prompt)
            .build()?
            .into()];
        let request = CreateChatCompletionRequestArgs::default()
            .model(self.source.to_string())
            .max_tokens(self.max_tokens)
            .messages(messages)
            .build()?;
        let mut stream = self.client.chat().create_stream(request).await?;

        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    response.choices.iter().for_each(|e| {
                        if let Some(ref content) = e.delta.content {
                            self.sender.send(content.clone()).unwrap();
                        }
                    });
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }

        Ok(())
    }
}
