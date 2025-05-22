use kalosm::language::*;
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use crate::{ai::SourceSize, error::*, Result};

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
pub struct Llma {
    model: Arc<Mutex<Chat>>,
}

impl Llma {
    pub async fn quick() -> Result<Self> {
        Self::new(SourceSize::Tiny).await
    }

    pub async fn new(size: SourceSize) -> Result<Self> {
        let source = match size {
            SourceSize::Tiny => LlamaSource::llama_3_2_1b_chat(),
            SourceSize::Small => LlamaSource::llama_7b_chat(),
            SourceSize::Base => LlamaSource::llama_8b_chat(),
            SourceSize::Medium => LlamaSource::llama_13b_chat(),
            SourceSize::Large => LlamaSource::llama_70b_chat(),
        };
        Self::with_source(source).await
    }

    pub async fn with_source(source: LlamaSource) -> Result<Self> {
        let model = Llama::builder().with_source(source).build().await?;
        let chat = Chat::builder(model).build();
        Ok(Llma {
            model: Arc::new(Mutex::new(chat)),
        })
    }

    pub fn with_model<T: Model>(model: T) -> Self
    where
        T::SyncModel: SyncModel + Send,
        <T::SyncModel as SyncModel>::Session: Send,
    {
        let chat = Chat::builder(model).build();
        Llma {
            model: Arc::new(Mutex::new(chat)),
        }
    }

    pub fn with_chat(chat: Chat) -> Self {
        Llma {
            model: Arc::new(Mutex::new(chat)),
        }
    }

    pub fn from_session<T: Model>(
        model: T,
        session: <<T as Model>::SyncModel as SyncModel>::Session,
    ) -> Self
    where
        T::SyncModel: SyncModel,
        <T::SyncModel as SyncModel>::Session: Send,
    {
        let chat = Chat::builder(model).with_session(session).build();
        Llma {
            model: Arc::new(Mutex::new(chat)),
        }
    }

    pub fn from_file<T: Model, P: AsRef<Path>>(model: T, path: P) -> Self
    where
        T::SyncModel: SyncModel,
        <T::SyncModel as SyncModel>::Session: Send,
    {
        let chat = Chat::builder(model).with_try_session_path(path).build();
        Llma {
            model: Arc::new(Mutex::new(chat)),
        }
    }

    pub fn prompt<T: AsRef<str>>(&self, prompt: T) -> Result<ChannelTextStream> {
        let prompt = prompt.as_ref();
        let prompt = if prompt.is_empty() { "\n>" } else { prompt };
        let prompt = prompt_input(prompt)?;
        if prompt.is_empty() {
            return Err(NoInputError.into());
        }
        let mut model = self.model.lock().unwrap();
        Ok(model.add_message(prompt))
    }

    pub fn load_session<T: Model, P: AsRef<Path>>(
        path: P,
    ) -> Result<<<T as Model>::SyncModel as SyncModel>::Session>
    where
        T::SyncModel: SyncModel,
        <T::SyncModel as SyncModel>::Session: Send,
    {
        <T::SyncModel as SyncModel>::Session::load_from(path).map_err(Into::into)
    }

    pub fn task_for<T: Parse + Schema + 'static>(
        &self,
        description: impl ToString,
    ) -> TaskBuilder<impl SendCreateParserState + Parser<Output = T> + 'static> {
        Task::builder_for::<T>(description)
    }

    pub fn history(&self) -> Vec<ChatHistoryItem> {
        self.model.lock().unwrap().history()
    }

    pub async fn save<T: AsRef<Path>>(&self, path: T) -> Result<()> {
        let mut model = self.model.lock().unwrap();
        model.save_session(path).await.map_err(Into::into)
    }
}
