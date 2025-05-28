pub mod llama;
pub mod openai;

use kalosm::{
    language::{
        ChatMessage, ChatModel, ChatSession, MessageType, TextCompletionBuilder,
        TextCompletionModelExt,
    },
    *,
};
use std::{
    fs,
    path::Path,
    sync::{Arc, Mutex},
};

use super::{ai::*, error::*, Result};

pub trait ChatModelType:
    ChatModel<
        Error: Send + Sync + std::error::Error + 'static,
        ChatSession: ChatSession<Error: std::error::Error + Send + Sync + 'static>
                         + Clone
                         + Send
                         + Sync
                         + 'static,
    > + TextCompletionModelExt
    + Clone
    + Send
    + Sync
    + 'static
{
}

impl<T> ChatModelType for T where
    T: ChatModel<
            Error: Send + Sync + std::error::Error + 'static,
            ChatSession: ChatSession<Error: std::error::Error + Send + Sync + 'static>
                             + Clone
                             + Send
                             + Sync
                             + 'static,
        > + TextCompletionModelExt
        + Clone
        + Send
        + Sync
        + 'static
{
}

pub trait ModelSource: Send + Sync + Clone {
    type Model: ChatModelType + Send + Sync + 'static;
    type Builder;

    fn default_size() -> SourceSize;
    fn builder() -> Self::Builder;
    async fn new() -> Result<Self::Model>;
    async fn create(size: SourceSize) -> Result<Self::Model>;
}

/**
A chatbot that can be used to interact with the model.
To use CUDA on your machine, follow these steps:

1. **Check GPU Compatibility**: Ensure your GPU supports CUDA. You can check this on the [NVIDIA CUDA GPUs page](https://developer.nvidia.com/cuda-gpus).

2. **Install NVIDIA Drivers**:
    - Update your package list:
      ```sh
      sudo apt update
      ```
    - Install the NVIDIA driver:
      ```sh
      sudo apt install nvidia-driver-470
      ```
    - Reboot your machine:
      ```sh
      sudo reboot
      ```

3. **Install CUDA Toolkit**:
    - Download the CUDA Toolkit from the [NVIDIA CUDA Toolkit page](https://developer.nvidia.com/cuda-downloads).
    - Follow the installation instructions provided on the download page for your specific Linux distribution.

4. **Set Up Environment Variables**:
    - Add the following lines to your `~/.bashrc` or `~/.zshrc` file:
      ```sh
      export PATH=/usr/local/cuda/bin:$PATH
      export LD_LIBRARY_PATH=/usr/local/cuda/lib64:$LD_LIBRARY_PATH
      ```
    - Source the updated file:
      ```sh
      source ~/.bashrc
      ```

5. **Verify Installation**:
    - Check the CUDA version:
      ```sh
      nvcc --version
      ```
    - Run a sample CUDA program to ensure everything is set up correctly. You can find sample programs in the CUDA installation directory, typically under `/usr/local/cuda/samples`.

6. **Install cuDNN (Optional but Recommended for Deep Learning)**:
    - Download cuDNN from the [NVIDIA cuDNN page](https://developer.nvidia.com/cudnn).
    - Follow the installation instructions provided on the download page.

After completing these steps, you should be able to use CUDA on your machine.
*/
#[derive(Clone)]
pub struct Chat<M: ChatModelType + Send + Sync + 'static> {
    model: Arc<Mutex<M>>,
}

impl<M: ChatModelType + Send + Sync + 'static> Chat<M>
where
    M::ChatSession: Send + Sync,
{
    /// Create a new instance with default/base size using a specific source type
    pub async fn quick<S: ModelSource<Model = M>>() -> Result<Self> {
        let model = S::new().await?;
        Ok(Self {
            model: Arc::new(Mutex::new(model)),
        })
    }

    /// Create a new instance with specified size using a specific source type
    pub async fn new<S: ModelSource<Model = M>>(size: SourceSize) -> Result<Self> {
        let model = S::create(size).await?;
        Ok(Self {
            model: Arc::new(Mutex::new(model)),
        })
    }

    /// Create instance with existing model
    pub fn with_model(model: M) -> Self {
        Self {
            model: Arc::new(Mutex::new(model)),
        }
    }

    /// Send a message and get response
    pub fn send<S: ModelSource<Model = M>, T: AsRef<str>>(
        &self,
        prompt: T,
    ) -> Result<TextCompletionBuilder<S::Model>> {
        let prompt = prompt.as_ref();
        let prompt = if prompt.is_empty() { "\n>" } else { prompt };
        let prompt = language::prompt_input(prompt)?;

        if prompt.is_empty() {
            return Err(NoInputError.into());
        }

        let model = self.model.lock().unwrap();
        let completion = model.complete(prompt);

        Ok(completion)
    }

    /// Load chat history from file
    pub async fn load_session<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = fs::read_to_string(path)?;
        let history: Vec<ChatMessage> = serde_json::from_str(&content)?;

        let mut llm = language::Llama::new().await?;
        let o: language::TextCompletionBuilder<language::Llama> = llm.complete(prompt);

        let mut chat = self.model.lock().unwrap();

        // Clear existing history and rebuild from saved data
        // Note: This assumes the Chat API allows rebuilding from history
        // You may need to adjust this based on the actual kalosm v0.4 Chat API
        for item in history {
            match item.role() {
                MessageType::UserMessage => {
                    // Add user message to history
                    // This is a placeholder - adjust based on actual Chat API
                }
                MessageType::ModelAnswer => {
                    // Add assistant message to history
                    // This is a placeholder - adjust based on actual Chat API
                }
                MessageType::SystemPrompt => {
                    // Add system message to history
                    // This is a placeholder - adjust based on actual Chat API
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Save current session to file
    pub async fn save_session<T: AsRef<Path>>(&self, path: T) -> Result<()> {
        let history = self.history().await?;
        let json = serde_json::to_string_pretty(&history)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Get current chat history
    pub async fn history(&self) -> Result<Vec<ChatHistoryItem>> {
        let chat = self.model.lock().unwrap();
        let mut history = Vec::new();

        // Example of how you might extract history:
        // if let Some(session) = chat.session() {
        //     for message in session.messages() {
        //         history.push(ChatHistoryItem {
        //             role: message.role().to_string(),
        //             content: message.content().to_string(),
        //             timestamp: Some(message.timestamp()),
        //         });
        //     }
        // }

        Ok(history)
    }
}
