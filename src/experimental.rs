use std::io::stdin;

use clap::Parser;
use miette::{bail, Context, IntoDiagnostic, Result};
use serde_json::{Map, Value};

use crate::Meilisearch;

#[derive(Debug, Parser)]
pub enum Experimental {
    /// Get the experimental features
    #[clap(aliases = &["list", "all"])]
    Get,
    /// Update the experimental features
    #[clap(aliases = &["post", "create"])]
    Update,
}

impl Experimental {
    pub fn execute(self, meili: Meilisearch) -> Result<()> {
        match self {
            Experimental::Get => meili.get_experimental_features(),
            Experimental::Update => meili.update_experimental_features(),
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
}
