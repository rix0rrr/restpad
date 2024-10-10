use std::{env, fs::File, io::Read};

use anyhow::{bail, Context};
use reqwest::{self, Url};

use crate::payload::Payload;

pub struct Navigator {
    current_url: reqwest::Url,
    current_page: Option<Payload>,
    history: Vec<reqwest::Url>,
    future: Vec<reqwest::Url>,
}

impl Navigator {
    /// Initializes a new browser
    ///
    /// The browser starts off with a base URL equal to the current working
    /// directory, so that it will be able to open relative files from the file
    /// system.
    pub fn new() -> anyhow::Result<Navigator> {
        let cwd = env::current_dir()?;
        let current_url = reqwest::Url::parse(&format!("file://{}/", cwd.as_path().display()))?;

        Ok(Navigator {
            current_url,
            current_page: None,
            history: Default::default(),
            future: Default::default(),
        })
    }

    /// Navigate to the given URL, returning its payload if successful
    pub async fn navigate(&mut self, url: &str) -> anyhow::Result<()> {
        let target_url = self.current_url.join(url)?;
        let old_url = self.do_navigate(target_url).await?;
        self.history.push(old_url);
        self.future.clear();
        Ok(())
    }

    async fn do_navigate(&mut self, target_url: reqwest::Url) -> anyhow::Result<reqwest::Url> {
        let payload: Payload = Self::load_url(target_url.clone()).await?;
        let old_url = std::mem::replace(&mut self.current_url, target_url);
        self.current_page = Some(payload);
        Ok(old_url)
    }

    pub fn has_history(&self) -> bool {
        // Bigger than 0 would have been logical, but the very first navigate
        // pushes something onto the history stack that doesn't count (because we initialize
        // with cwd as the initial url to relative file names work out).
        self.history.len() > 1
    }

    pub fn has_future(&self) -> bool {
        !self.future.is_empty()
    }

    pub async fn back(&mut self) -> anyhow::Result<()> {
        if !self.has_history() {
            return Ok(());
        }
        let Some(prev) = self.history.pop() else {
            return Ok(());
        };
        let old_url = self.do_navigate(prev).await?;
        self.future.push(old_url);
        Ok(())
    }

    pub async fn forward(&mut self) -> anyhow::Result<()> {
        let Some(next) = self.future.pop() else {
            return Ok(());
        };
        let old_url = self.do_navigate(next).await?;
        self.history.push(old_url);
        Ok(())
    }

    pub async fn refresh(&mut self) -> anyhow::Result<()> {
        self.do_navigate(self.current_url.clone()).await?;
        Ok(())
    }

    /// Reads the current page's payload
    pub fn current(&self) -> Option<&Payload> {
        self.current_page.as_ref()
    }

    /// Loads a given URL, handling it specially if it is a local file
    async fn load_url(url: Url) -> anyhow::Result<Payload> {
        if url.scheme() == "file" {
            let Ok(file_path) = url.to_file_path() else {
                bail!("Not a valid file path: {:?}", url);
            };
            let mut file = File::open(&file_path)
                .with_context(|| format!("Failed to open {}", file_path.display()))?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            let payload: Payload = serde_json::from_str(&contents)?;
            Ok(payload)
        } else {
            let response = reqwest::get(url).await?;
            let payload: Payload = response.json().await?;
            Ok(payload)
        }
    }
}
