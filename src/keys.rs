use std::io::stdin;

use clap::Parser;
use miette::{bail, miette, IntoDiagnostic, Result};
use serde_json::{json, Map, Value};

use crate::Meilisearch;

#[derive(Debug, Parser)]
pub enum Key {
    /// List all keys.
    #[clap(aliases = &["all"])]
    List,
    /// Get a key, by default use the key provided by `-k`.
    Get {
        /// The key you want to retrieve.
        k: Option<String>,
    },
    /// Create a key. The json needs to be piped in the command.
    #[clap(aliases = &["post"])]
    Create,
    /// Update a key. The json needs to be piped in the command.
    #[clap(aliases = &["patch"])]
    Update {
        /// The key you want to update. If you don't provide
        /// it here you need to send it in the json.
        k: Option<String>,
    },
    /// Delete a key.
    Delete {
        /// The key you want to delete.
        k: String,
    },
    /// Show an example of a valid json you can send to create a key.
    Template,
}

impl Key {
    pub fn execute(self, meili: Meilisearch) -> Result<()> {
        match self {
            Key::List => meili.get_keys(),
            Key::Get { k } => meili.get_key(k),
            Key::Create => meili.create_key(),
            Key::Update { k } => meili.update_key(k),
            Key::Delete { k } => meili.delete_key(k),
            Key::Template => meili.template(),
        }
    }
}

impl Meilisearch {
    fn get_keys(&self) -> Result<()> {
        let response = self
            .get(format!("{}/keys", self.addr))
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }

    fn get_key(&self, key: Option<String>) -> Result<()> {
        if let Some(key) = key.or_else(|| self.key.clone()) {
            let response = self
                .get(format!("{}/keys/{}", self.addr, key))
                .send()
                .into_diagnostic()?;
            self.handle_response(response)
        } else {
            bail!("No key to retrieve")
        }
    }

    fn create_key(&self) -> Result<()> {
        if atty::isnt(atty::Stream::Stdin) {
            let value: Map<String, Value> = serde_json::from_reader(stdin()).into_diagnostic()?;
            let response = self
                .post(format!("{}/keys", self.addr))
                .json(&value)
                .send()
                .into_diagnostic()?;
            self.handle_response(response)
        } else {
            bail!("You need to send a key. See `mieli template`.")
        }
    }

    fn update_key(&self, key: Option<String>) -> Result<()> {
        if atty::isnt(atty::Stream::Stdin) {
            let value: Map<String, Value> = serde_json::from_reader(stdin()).into_diagnostic()?;
            let key = key.as_deref().or(value["key"].as_str()).ok_or(miette!(
                "You need to provide a key either in the json or as an argument"
            ))?;
            let response = self
                .patch(format!("{}/keys/{}", self.addr, key))
                .json(&value)
                .send()
                .into_diagnostic()?;
            self.handle_response(response)
        } else {
            bail!("You need to send a key. See `mieli template`.")
        }
    }

    fn delete_key(&self, key: String) -> Result<()> {
        let response = self
            .delete(format!("{}/keys/{}", self.addr, key))
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }

    fn template(&self) -> Result<()> {
        let json = json!({
          "description": "Add documents key",
          "actions": ["documents.add"],
          "indexes": ["mieli"],
          "expiresAt": "2021-11-13T00:00:00Z"
        });
        println!(
            "{}",
            colored_json::to_colored_json_auto(&json).into_diagnostic()?
        );
        Ok(())
    }
}
