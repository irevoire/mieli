use clap::Parser;
use miette::{IntoDiagnostic, Result};
use serde_json::json;

use crate::Meilisearch;

#[derive(Debug, Parser)]
pub enum IndexesCommand {
    /// List all indexes.
    #[clap(aliases = &["all"])]
    List,
    /// Get an index, by default use the index provided by `-i`.
    Get {
        /// The index you want to retrieve.
        #[clap(name = "idx")]
        index: Option<String>,
    },
    /// Create an index, by default use the index provided by `-i`.
    Create {
        /// The index you want to create.
        #[clap(name = "idx")]
        index: Option<String>,
        /// Primary key
        #[clap(short, long)]
        primary: Option<String>,
    },
    /// Update an index, by default use the index provided by `-i`.
    Update {
        /// The index you want to update.
        #[clap(name = "idx")]
        index: Option<String>,
        /// Primary key
        #[clap(short, long)]
        primary: Option<String>,
    },
    /// Delete an index, by default use the index provided by `-i`.
    Delete {
        /// The index you want to delete.
        #[clap(name = "idx")]
        index: Option<String>,
    },
}

impl IndexesCommand {
    pub async fn execute(self, meili: Meilisearch) -> Result<()> {
        match self {
            IndexesCommand::List => meili.get_all_indexes().await,
            IndexesCommand::Get { index } => meili.get_index(index).await,
            IndexesCommand::Create { index, primary } => meili.create_index(index, primary).await,
            IndexesCommand::Update { index, primary } => meili.update_index(index, primary).await,
            IndexesCommand::Delete { index } => meili.delete_index(index).await,
        }
    }
}

impl Meilisearch {
    async fn get_all_indexes(&self) -> Result<()> {
        let response = self
            .get(format!("{}/indexes", self.addr))
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }

    async fn get_index(&self, index: Option<String>) -> Result<()> {
        let index = index.unwrap_or_else(|| self.index.to_string());
        let response = self
            .get(format!("{}/indexes/{}", self.addr, index))
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }

    async fn create_index(&self, index: Option<String>, primary_key: Option<String>) -> Result<()> {
        let index = index.unwrap_or_else(|| self.index.to_string());
        let mut body = json!({ "uid": index });
        if let Some(primary_key) = primary_key {
            body["primaryKey"] = json!(primary_key);
        }
        let response = self
            .post(format!("{}/indexes", self.addr))
            .json(&body)
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }

    async fn update_index(&self, index: Option<String>, primary_key: Option<String>) -> Result<()> {
        let index = index.unwrap_or_else(|| self.index.to_string());
        let mut body = json!({});
        if let Some(primary_key) = primary_key {
            body["primaryKey"] = json!(primary_key);
        }
        let url = format!("{}/indexes/{}", self.addr, index);
        let mut response = self.patch(&url).json(&body).send().into_diagnostic()?;
        if response.status().as_u16() == 405 {
            response = self.post(url).send().into_diagnostic()?;
        }
        self.handle_response(response)
    }

    async fn delete_index(&self, index: Option<String>) -> Result<()> {
        let index = index.unwrap_or_else(|| self.index.to_string());
        let response = self
            .delete(format!("{}/indexes/{}", self.addr, index))
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }
}
