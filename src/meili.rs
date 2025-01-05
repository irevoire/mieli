use std::{
    env,
    fmt::Display,
    io::{stdout, BufWriter, Write},
    path::Path,
};

use clap::{CommandFactory, Parser};
use clap_complete::{
    generate,
    shells::{Bash, Elvish, Fish, Zsh},
};
use dialoguer::Confirm;
use miette::{bail, miette, Context, IntoDiagnostic, Result};

use crate::options::Options;

#[derive(Debug, Parser)]
pub enum Inner {
    /// Generate the autocomplete file for your shell.
    AutoComplete { shell: Option<String> },
    /// Download and install the latest `mieli` version.
    Upgrade,
    /// Return the current version of mieli.
    Version,
}
