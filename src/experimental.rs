use std::{io::stdin, process::Command};

use clap::Parser;
use log::warn;
use miette::{bail, Context, IntoDiagnostic, Result};
use serde_json::{Map, Value};

use crate::{format, Meilisearch};

#[derive(Debug, Parser)]
pub enum Experimental {
    /// Get the experimental features
    #[clap(aliases = &["list", "all"])]
    Get,
    /// Update the experimental features
    #[clap(aliases = &["post", "create"])]
    Update {
        /// Interactively update the experimental features
        #[clap(long, aliases = &["int"])]
        interactive: bool,
    },
}

impl Experimental {
    pub fn execute(self, meili: Meilisearch) -> Result<()> {
        match self {
            Experimental::Get => meili.get_experimental_features(),
            Experimental::Update { interactive: false } => meili.update_experimental_features(),
            Experimental::Update { interactive: true } => {
                meili.interactive_update_experimental_features()
            }
        }
    }
}

impl Meilisearch {
    fn get_experimental_features(&self) -> Result<()> {
        let response = self
            .get(format!("{}/experimental-features", self.addr))
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }

    fn update_experimental_features(&self) -> Result<()> {
        if atty::isnt(atty::Stream::Stdin) {
            let value: Map<String, Value> = serde_json::from_reader(stdin())
                .into_diagnostic()
                .context("Could not deserialize stdin as json")?;

            let response = self
                .patch(format!("{}/experimental-features", self.addr))
                .json(&value)
                .send()
                .into_diagnostic()?;
            self.handle_response(response)
        } else {
            bail!("You need to send a key. See `mieli template`.")
        }
    }

    fn interactive_update_experimental_features(&self) -> Result<()> {
        let response = self
            .get(format!("{}/experimental-features", self.addr))
            .send()
            .into_diagnostic()?;
        let features = format::write_response_full(response, self.verbose)?;
        let mut tempfile = tempfile::Builder::new()
            .suffix(".json")
            .tempfile()
            .into_diagnostic()?;
        serde_json::to_writer_pretty(&mut tempfile, &features)
            .into_diagnostic()
            .context("Could not write the feature in a tempfile")?;
        let path = tempfile.into_temp_path();

        let editor = match std::env::var("EDITOR") {
            Ok(editor) => editor,
            Err(std::env::VarError::NotPresent) => "vi".to_string(),
            Err(e) => {
                warn!("Cannot read the `$EDITOR` env variable. `vi` will be used: {e}");
                "vi".to_string()
            }
        };

        let ret = Command::new(&editor)
            .arg(path.as_os_str())
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .output();
        if let Err(err) = ret {
            warn!(
                "Editor `{}` failed to edit the file at the path `{}`: {err}",
                editor,
                path.to_string_lossy()
            );
            Err(err).into_diagnostic()?;
        }
        let bytes = std::fs::read(path).into_diagnostic()?;
        let value: Map<String, Value> = serde_json::from_slice(&bytes)
            .into_diagnostic()
            .context("Could not deserialize the payload as json")?;

        let response = self
            .patch(format!("{}/experimental-features", self.addr))
            .json(&value)
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }
}
