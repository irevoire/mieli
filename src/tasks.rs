use clap::Parser;
use miette::{IntoDiagnostic, Result};
use serde::Serialize;

use crate::Meilisearch;

#[derive(Debug, Parser)]
pub enum TasksCommand {
    /// Get tasks
    ///
    /// List all tasks globally, regardless of index. The task objects are contained in the results array.
    /// Tasks are always returned in descending order of uid. This means that by default, the most recently created task objects appear first.
    /// Task results are paginated and can be filtered.
    #[clap(aliases = &["l", "get", "g"])]
    List {
        #[clap(flatten)]
        params: TaskListParameters,
        /// Get a single task. Filter cannot be used if an id is specified
        id: Option<u32>,
    },
    /// Cancel tasks
    ///
    /// Cancel any number of enqueued or processing tasks based on their uid, status, type, indexUid, or the date at which they were enqueued (enqueuedAt) or processed (startedAt).
    /// Task cancellation is an atomic transaction: either all tasks are successfully canceled or none are.
    Cancel(TaskFilter),
    /// Delete tasks
    ///
    /// Delete a finished (succeeded, failed, or canceled) task based on uid, status, type, indexUid, canceledBy, or date. Task deletion is an atomic transaction: either all tasks are successfully deleted, or none are.
    #[clap(aliases = &["d", "remove", "rm", "r"])]
    Delete(TaskFilter),
}

#[derive(Debug, Default, PartialEq, Eq, Parser, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskListParameters {
    #[clap(flatten)]
    #[serde(flatten)]
    pagination: TaskPagination,
    #[clap(flatten)]
    #[serde(flatten)]
    filter: TaskFilter,
}

#[derive(Debug, Default, PartialEq, Eq, Parser, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskPagination {
    /// Number of tasks to return
    #[clap(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u32>,
    /// `uid` of the first task returned
    #[clap(long, aliases = &["offset"])]
    #[serde(skip_serializing_if = "Option::is_none")]
    from: Option<u32>,
    /// If true, returns results in the reverse order, from oldest to most recent
    #[clap(long, aliases = &["rev"])]
    #[serde(skip_serializing_if = "Option::is_none")]
    reverse: Option<bool>,
}

#[derive(Debug, Default, PartialEq, Eq, Parser, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskFilter {
    /// Filter tasks by their uid. Separate multiple task uids with a comma (,)
    #[clap(long, aliases = &["uid", "id"])]
    #[serde(skip_serializing_if = "Option::is_none")]
    uids: Option<String>,
    /// Filter tasks by their batchUid. Separate multiple batchUids with a comma (,)
    #[clap(long, aliases = &["batchUids", "batches", "batch"])]
    #[serde(skip_serializing_if = "Option::is_none")]
    batch_uids: Option<String>,
    /// Filter tasks by their status. Separate multiple task statuses with a comma (,)
    #[clap(long, aliases = &["status"])]
    #[serde(skip_serializing_if = "Option::is_none")]
    statuses: Option<String>,
    /// Filter tasks by their type. Separate multiple task types with a comma (,)
    #[clap(long, aliases = &["type"])]
    #[serde(skip_serializing_if = "Option::is_none")]
    types: Option<String>,
    /// Filter tasks by their indexUid. Separate multiple task indexUids with a comma (,). Case-sensitive
    #[clap(long, aliases = &["indexes", "indexUids", "indexUid"])]
    #[serde(skip_serializing_if = "Option::is_none")]
    index_uids: Option<String>,
    /// Filter tasks by their canceledBy field. Separate multiple task uids with a comma (,)
    #[clap(long, aliases = &["canceledBy"])]
    #[serde(skip_serializing_if = "Option::is_none")]
    canceled_by: Option<String>,
    /// Filter tasks by their `enqueuedAt` field
    #[clap(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    before_enqueued_at: Option<String>,
    /// Filter tasks by their `enqueuedAt` field
    #[clap(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    after_enqueued_at: Option<String>,
    /// Filter tasks by their `startedAt` field
    #[clap(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    before_started_at: Option<String>,
    /// Filter tasks by their `startedAt` field
    #[clap(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    after_started_at: Option<String>,
    /// Filter tasks by their `finishedAt` field
    #[clap(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    before_finished_at: Option<String>,
    /// Filter tasks by their `finishedAt` field
    #[clap(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    after_finished_at: Option<String>,
}

impl TasksCommand {
    pub fn execute(self, meili: Meilisearch) -> Result<()> {
        match self {
            TasksCommand::List { params, id: None } => meili.get_tasks(params),
            TasksCommand::List {
                params,
                id: Some(id),
            } => {
                if params != TaskListParameters::default() {
                    log::warn!("extra parameters have been specified while retrieving a task by id. The following parameters will be ignored: `{}`", yaup::to_string(&params).unwrap());
                }
                meili.get_task(id)
            }
            TasksCommand::Cancel(filter) => meili.cancel_tasks(filter),
            TasksCommand::Delete(filter) => meili.delete_tasks(filter),
        }
    }
}

impl Meilisearch {
    fn get_task(&self, id: u32) -> Result<()> {
        let response = self
            .get(format!("{}/tasks/{}", self.addr, id))
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }

    fn get_tasks(&self, params: TaskListParameters) -> Result<()> {
        let response = self
            .get(format!(
                "{}/tasks{}",
                self.addr,
                yaup::to_string(&params).into_diagnostic()?
            ))
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }

    fn cancel_tasks(&self, filter: TaskFilter) -> Result<()> {
        let response = self
            .post(format!(
                "{}/tasks/cancel{}",
                self.addr,
                yaup::to_string(&filter).into_diagnostic()?
            ))
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }

    fn delete_tasks(&self, filter: TaskFilter) -> Result<()> {
        let response = self
            .delete(format!(
                "{}/tasks{}",
                self.addr,
                yaup::to_string(&filter).into_diagnostic()?
            ))
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }
}
