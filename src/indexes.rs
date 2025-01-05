use clap::Parser;
use miette::{IntoDiagnostic, Result};
use serde::Serialize;
use serde_json::json;

use crate::Meilisearch;

#[derive(Debug, Parser, Serialize)]
pub struct ListIndexes {
    /// Number of indexes to skip
    #[clap(long)]
    offset: Option<u32>,
    /// Number of indexes to return
    #[clap(long)]
    limit: Option<u32>,
}

#[derive(Debug, Parser)]
pub enum IndexesCommand {
    /// List all indexes.
    #[clap(aliases = &["all"])]
    List(ListIndexes),
    /// Get an index, by default use the index provided by `-i`.
    Get {
        /// The index you want to retrieve.
        #[clap(aliases = &["idx", "uid", "index_uid", "indexUid"])]
        index: Option<String>,
    },
    /// Create an index, by default use the index provided by `-i`.
    Create {
        /// The index you want to create.
        #[clap(aliases = &["idx", "uid", "index_uid", "indexUid"])]
        index: Option<String>,
        /// Primary key
        #[clap(short, long, aliases = &["primary-key", "primary_key", "primaryKey", "pk"])]
        primary: Option<String>,
    },
    /// Update an index, by default use the index provided by `-i`.
    Update {
        /// The index you want to update.
        #[clap(aliases = &["idx", "uid", "index_uid", "indexUid"])]
        index: Option<String>,
        /// Primary key
        #[clap(short, long, aliases = &["primary-key", "primary_key", "primaryKey", "pk"])]
        primary: Option<String>,
    },
    /// Delete an index, by default use the index provided by `-i`.
    Delete {
        /// The index you want to delete.
        #[clap(aliases = &["idx", "uid", "index_uid", "indexUid"])]
        index: Option<String>,
    },
}

impl IndexesCommand {
    pub fn execute(self, meili: Meilisearch) -> Result<()> {
        match self {
            IndexesCommand::List(opt) => meili.get_all_indexes(opt),
            IndexesCommand::Get { index } => meili.get_index(index),
            IndexesCommand::Create { index, primary } => meili.create_index(index, primary),
            IndexesCommand::Update { index, primary } => meili.update_index(index, primary),
            IndexesCommand::Delete { index } => meili.delete_index(index),
        }
    }
}

impl Meilisearch {
    fn get_all_indexes(&self, opt: ListIndexes) -> Result<()> {
        let url = format!("{}/indexes{}", self.addr, yaup::to_string(&opt).unwrap());
        let response = self.get(url).send().into_diagnostic()?;
        self.handle_response(response)
    }

    fn get_index(&self, index: Option<String>) -> Result<()> {
        let index = index.unwrap_or_else(|| self.index.to_string());
        let response = self
            .get(format!("{}/indexes/{}", self.addr, index))
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }

    fn create_index(&self, index: Option<String>, primary_key: Option<String>) -> Result<()> {
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

    fn update_index(&self, index: Option<String>, primary_key: Option<String>) -> Result<()> {
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

    fn delete_index(&self, index: Option<String>) -> Result<()> {
        let index = index.unwrap_or_else(|| self.index.to_string());
        let response = self
            .delete(format!("{}/indexes/{}", self.addr, index))
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }
}
