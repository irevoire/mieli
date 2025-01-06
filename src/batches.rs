use clap::Parser;
use miette::{IntoDiagnostic, Result};

use crate::{tasks::TaskListParameters, Meilisearch};

#[derive(Debug, Parser)]
pub enum BatchesCommand {
    /// Get batches
    #[clap(aliases = &["l", "get", "g"])]
    List {
        #[clap(flatten)]
        params: TaskListParameters,
        /// Get a single batch. Filter cannot be used if an id is specified
        id: Option<u32>,
    },
}

impl BatchesCommand {
    pub fn execute(self, meili: Meilisearch) -> Result<()> {
        match self {
            BatchesCommand::List { params, id: None } => meili.get_batches(params),
            BatchesCommand::List {
                params,
                id: Some(id),
            } => {
                if params != TaskListParameters::default() {
                    log::warn!("extra parameters have been specified while retrieving a task by id. The following parameters will be ignored: `{}`", yaup::to_string(&params).unwrap());
                }
                meili.get_batch(id)
            }
        }
    }
}

impl Meilisearch {
    fn get_batch(&self, id: u32) -> Result<()> {
        let response = self
            .get(format!("{}/batches/{}", self.addr, id))
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }

    fn get_batches(&self, params: TaskListParameters) -> Result<()> {
        let response = self
            .get(format!(
                "{}/batches{}",
                self.addr,
                yaup::to_string(&params).into_diagnostic()?
            ))
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }
}
