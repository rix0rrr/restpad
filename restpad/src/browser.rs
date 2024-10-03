use std::{env, fs::File, io::Read};

use anyhow::bail;
use reqwest::{self, Url};

use crate::payload::Payload;

/// Information about the current launchpad
pub struct LaunchPadInfo {
    /// The model of the launchpad
    model: String,
    /// How many buttons it has in the X dimension
    width: u32,
    /// How many buttons it has in the Y dimension
    height: u32,
}

pub struct Browser {
    current_url: reqwest::Url,
    current_page: Option<Payload>,
}

impl Browser {
    /// Initializes a new browser
    ///
    /// The browser starts off with a base URL equal to the current working
    /// directory, so that it will be able to open relative files from the file
    /// system.
    pub fn new() -> anyhow::Result<Browser> {
        let cwd = env::current_dir()?;
        let current_url = reqwest::Url::parse(&format!("file://{}/", cwd.as_path().display()))?;

        Ok(Browser {
            current_url,
            current_page: None,
        })
    }

    /// Navigate to the given URL, returning its payload if successful
    pub async fn navigate(&mut self, url: &str) -> anyhow::Result<&Payload> {
        let target_url = self.current_url.join(url)?;
        let payload: Payload = Self::load_url(target_url.clone()).await?;

        self.current_url = target_url;
        self.current_page = Some(payload);

        self.current_page
            .as_ref()
            .ok_or(anyhow::anyhow!("No page loaded"))
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
            let mut file = File::open(file_path)?;
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
