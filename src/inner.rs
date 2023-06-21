use std::{
    env,
    fmt::Display,
    fs,
    io::{stdout, BufWriter, Write},
    path::{Path, PathBuf},
};

use byte_unit::Byte;
use clap::{CommandFactory, Parser};
use clap_complete::{
    generate,
    shells::{Bash, Elvish, Fish, Zsh},
};
use dialoguer::Confirm;
use heed::{types::ByteSlice, EnvOpenOptions, PolyDatabase, RoTxn};
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
    /// Print the index stats (sizes, number of entries, etc.)
    Stats { path: PathBuf },
}

impl Inner {
    pub fn execute(self) -> Result<()> {
        match self {
            Inner::Upgrade => upgrade(),
            Inner::AutoComplete { shell } => auto_complete(shell),
            Inner::Version => version(),
            Inner::Stats { path } => stats(path),
        }
    }
}

pub fn upgrade() -> Result<()> {
    let github = "https://github.com";
    let latest_release = reqwest::blocking::get(format!("{github}/irevoire/mieli/releases/latest"))
        .into_diagnostic()?;
    let latest_release_url = format!("{github}{}", latest_release.url().path());

    let mut latest_release = latest_release_url.rsplit_once('/').unwrap().1.to_string();
    if latest_release.starts_with('v') {
        latest_release = latest_release.chars().skip(1).collect();
    }
    let current_version = env!("CARGO_PKG_VERSION");

    if current_version >= latest_release.as_str() {
        println!("Current version {current_version} is equal or higher than latest published version {latest_release}");
        return Ok(());
    }

    let executable_path = env::current_exe()
        .into_diagnostic()
        .with_context(|| "can't get the executable path")?;

    #[allow(unused)]
    let mut bin_name: Result<&str> = Err(miette!("Could not determine the right binary for your OS / architecture.\nYou can check the latest release here: {latest_release}."));

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        bin_name = Ok("mieli-linux-amd64");
    }
    #[cfg(all(target_os = "macos", target_arch = "amd64"))]
    {
        bin_name = Ok("mieli-macos-amd64");
    }
    let bin_url = format!(
        "{github}/irevoire/mieli/releases/download/v{latest_release}/{}",
        bin_name?
    );
    let mut executable_dir = executable_path.clone();
    executable_dir.pop();
    let mut tmp = tempfile::NamedTempFile::new_in(executable_dir).into_diagnostic()?;
    let mut res = reqwest::blocking::get(bin_url).into_diagnostic()?;
    res.copy_to(&mut tmp).into_diagnostic()?;

    let file = tmp
        .persist(&executable_path)
        .into_diagnostic()
        .with_context(|| {
            format!(
                "Error while trying to write the binary to {:?}",
                executable_path
            )
        })?;

    let mut permissions = file.metadata().into_diagnostic()?.permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        #[allow(clippy::non_octal_unix_permissions)]
        //                     rwxrwxrwx
        permissions.set_mode(0b111101101);
    }

    file.set_permissions(permissions).into_diagnostic()?;
    Ok(())
}

pub fn version() -> Result<()> {
    println!(
        "{} - version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    Ok(())
}

#[derive(Debug, Copy, Clone)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    Elvish,
}

impl Shell {
    pub fn generator(&self, writer: impl Write) {
        let mut writer = BufWriter::new(writer);
        let mut opt = Options::command_for_update();

        match self {
            Self::Bash => generate(Bash, &mut opt, env!("CARGO_BIN_NAME"), &mut writer),
            Self::Zsh => generate(Zsh, &mut opt, env!("CARGO_BIN_NAME"), &mut writer),
            Self::Fish => generate(Fish, &mut opt, env!("CARGO_BIN_NAME"), &mut writer),
            Self::Elvish => generate(Elvish, &mut opt, env!("CARGO_BIN_NAME"), &mut writer),
        }
    }

    pub fn completion_path(&self) -> Result<String> {
        let home = std::env::var("HOME").into_diagnostic()?;
        Ok(match self {
            Shell::Bash => format!("{home}/.bash_completion.d/mieli-completion.bash",),
            Shell::Zsh => format!("{home}/.zfunc/_mieli",),
            Shell::Fish => format!("{home}/.config/fish/completions/mieli.fish"),
            Shell::Elvish => bail!("I don't know where the elvish completion files are supposed to be. If you use elvish please submit an issue."),
        })
    }

    pub fn install_completion(&self) -> Result<()> {
        let path = self.completion_path()?;
        let file_path = Path::new(&path);
        let dir_path = file_path
            .parent()
            .expect("Internal: Can't access to the directory");

        if !dir_path.exists() {
            std::fs::create_dir_all(dir_path).into_diagnostic()?;
        }

        let writer = std::fs::File::create(file_path).into_diagnostic()?;
        self.generator(writer);

        println!("Done!");

        // TODO: I should check if I need to push the path in the bashrc / zshrc
        Ok(())
    }
}

impl Display for Shell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Shell::Bash => write!(f, "bash"),
            Shell::Zsh => write!(f, "zsh"),
            Shell::Fish => write!(f, "fish"),
            Shell::Elvish => write!(f, "elvish"),
        }
    }
}

pub fn auto_complete(shell: Option<String>) -> Result<()> {
    if let Some(shell) = shell.or_else(|| std::env::var("SHELL").ok()) {
        let shell = match shell.rsplit('/').next() {
            Some("bash") => Shell::Bash,
            Some("zsh") => Shell::Zsh,
            Some("fish") => Shell::Fish,
            Some("elvish") => Shell::Elvish,
            _ => bail!("Unsupported shell `{}`.", shell),
        };

        if atty::is(atty::Stream::Stdout) {
            let path = shell.completion_path()?;
            println!("Infered the shell `{shell}`. If this is wrong you can give your shell as argument.");
            println!("For {shell} the completion must be installed in `{path}`.",);
            println!(
                "The following command must be executed to enable the autocompletion of commands;"
            );
            println!(
                "{} self auto-complete --{shell} > {path}",
                env!("CARGO_BIN_NAME"),
            );
            if Confirm::new()
                .with_prompt("Do you want me to install it for you?")
                .interact()
                .into_diagnostic()?
            {
                shell.install_completion()?;
            }
        } else {
            shell.generator(stdout());
        }
    } else {
        bail!("Can't detect your shell. Env variable $SHELL is not set.");
    }

    Ok(())
}

/// List of the indexes
pub const MAIN: &str = "main";
pub const WORD_DOCIDS: &str = "word-docids";
pub const EXACT_WORD_DOCIDS: &str = "exact-word-docids";
pub const WORD_PREFIX_DOCIDS: &str = "word-prefix-docids";
pub const EXACT_WORD_PREFIX_DOCIDS: &str = "exact-word-prefix-docids";
pub const WORD_PAIR_PROXIMITY_DOCIDS: &str = "word-pair-proximity-docids";
pub const WORD_PREFIX_PAIR_PROXIMITY_DOCIDS: &str = "word-prefix-pair-proximity-docids";
pub const PREFIX_WORD_PAIR_PROXIMITY_DOCIDS: &str = "prefix-word-pair-proximity-docids";
pub const WORD_POSITION_DOCIDS: &str = "word-position-docids";
pub const WORD_FIELD_ID_DOCIDS: &str = "word-field-id-docids";
pub const WORD_PREFIX_POSITION_DOCIDS: &str = "word-prefix-position-docids";
pub const WORD_PREFIX_FIELD_ID_DOCIDS: &str = "word-prefix-field-id-docids";
pub const FIELD_ID_WORD_COUNT_DOCIDS: &str = "field-id-word-count-docids";
pub const FACET_ID_F64_DOCIDS: &str = "facet-id-f64-docids";
pub const FACET_ID_EXISTS_DOCIDS: &str = "facet-id-exists-docids";
pub const FACET_ID_IS_NULL_DOCIDS: &str = "facet-id-is-null-docids";
pub const FACET_ID_IS_EMPTY_DOCIDS: &str = "facet-id-is-empty-docids";
pub const FACET_ID_STRING_DOCIDS: &str = "facet-id-string-docids";
pub const FIELD_ID_DOCID_FACET_F64S: &str = "field-id-docid-facet-f64s";
pub const FIELD_ID_DOCID_FACET_STRINGS: &str = "field-id-docid-facet-strings";
pub const VECTOR_ID_DOCID: &str = "vector-id-docids";
pub const DOCUMENTS: &str = "documents";
pub const SCRIPT_LANGUAGE_DOCIDS: &str = "script_language_docids";

#[derive(Debug)]
pub struct Stats {
    pub number_of_entries: u64,
    pub size_of_keys: u64,
    pub size_of_data: u64,
    pub size_of_entries: u64,
}

fn compute_stats(rtxn: &RoTxn, db: PolyDatabase) -> Result<Stats> {
    let mut number_of_entries = 0;
    let mut size_of_keys = 0;
    let mut size_of_data = 0;

    for result in db.iter::<ByteSlice, ByteSlice>(rtxn).unwrap() {
        let (key, data) = result.unwrap();
        number_of_entries += 1;
        size_of_keys += key.len() as u64;
        size_of_data += data.len() as u64;
    }

    Ok(Stats {
        number_of_entries,
        size_of_keys,
        size_of_data,
        size_of_entries: size_of_keys + size_of_data,
    })
}

fn get_folder_size(path: &Path) -> Result<u64, std::io::Error> {
    let mut total_size = 0;

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let entry_path = entry.path();

        if file_type.is_file() {
            total_size += fs::metadata(entry_path)?.len();
        } else if file_type.is_dir() {
            total_size += get_folder_size(&entry_path)?;
        }
    }

    Ok(total_size)
}

pub fn stats(path: PathBuf) -> Result<()> {
    let folder_size = get_folder_size(&path).unwrap();
    let byte = Byte::from_bytes(folder_size);
    let adjusted_byte = byte.get_appropriate_unit(false);
    println!("total - {}", adjusted_byte.to_string());

    let env = EnvOpenOptions::new().max_dbs(24).open(path).unwrap();

    let mut wtxn = env.write_txn().unwrap();
    let main = env.create_poly_database(&mut wtxn, Some(MAIN)).unwrap();
    let word_docids = env
        .create_poly_database(&mut wtxn, Some(WORD_DOCIDS))
        .unwrap();
    let exact_word_docids = env
        .create_poly_database(&mut wtxn, Some(EXACT_WORD_DOCIDS))
        .unwrap();
    let word_prefix_docids = env
        .create_poly_database(&mut wtxn, Some(WORD_PREFIX_DOCIDS))
        .unwrap();
    let exact_word_prefix_docids = env
        .create_poly_database(&mut wtxn, Some(EXACT_WORD_PREFIX_DOCIDS))
        .unwrap();
    let word_pair_proximity_docids = env
        .create_poly_database(&mut wtxn, Some(WORD_PAIR_PROXIMITY_DOCIDS))
        .unwrap();
    let script_language_docids = env
        .create_poly_database(&mut wtxn, Some(SCRIPT_LANGUAGE_DOCIDS))
        .unwrap();
    let word_prefix_pair_proximity_docids = env
        .create_poly_database(&mut wtxn, Some(WORD_PREFIX_PAIR_PROXIMITY_DOCIDS))
        .unwrap();
    let prefix_word_pair_proximity_docids = env
        .create_poly_database(&mut wtxn, Some(PREFIX_WORD_PAIR_PROXIMITY_DOCIDS))
        .unwrap();
    let word_position_docids = env
        .create_poly_database(&mut wtxn, Some(WORD_POSITION_DOCIDS))
        .unwrap();
    let word_fid_docids = env
        .create_poly_database(&mut wtxn, Some(WORD_FIELD_ID_DOCIDS))
        .unwrap();
    let field_id_word_count_docids = env
        .create_poly_database(&mut wtxn, Some(FIELD_ID_WORD_COUNT_DOCIDS))
        .unwrap();
    let word_prefix_position_docids = env
        .create_poly_database(&mut wtxn, Some(WORD_PREFIX_POSITION_DOCIDS))
        .unwrap();
    let word_prefix_fid_docids = env
        .create_poly_database(&mut wtxn, Some(WORD_PREFIX_FIELD_ID_DOCIDS))
        .unwrap();
    let facet_id_f64_docids = env
        .create_poly_database(&mut wtxn, Some(FACET_ID_F64_DOCIDS))
        .unwrap();
    let facet_id_string_docids = env
        .create_poly_database(&mut wtxn, Some(FACET_ID_STRING_DOCIDS))
        .unwrap();
    let facet_id_exists_docids = env
        .create_poly_database(&mut wtxn, Some(FACET_ID_EXISTS_DOCIDS))
        .unwrap();
    let facet_id_is_null_docids = env
        .create_poly_database(&mut wtxn, Some(FACET_ID_IS_NULL_DOCIDS))
        .unwrap();
    let facet_id_is_empty_docids = env
        .create_poly_database(&mut wtxn, Some(FACET_ID_IS_EMPTY_DOCIDS))
        .unwrap();
    let field_id_docid_facet_f64s = env
        .create_poly_database(&mut wtxn, Some(FIELD_ID_DOCID_FACET_F64S))
        .unwrap();
    let field_id_docid_facet_strings = env
        .create_poly_database(&mut wtxn, Some(FIELD_ID_DOCID_FACET_STRINGS))
        .unwrap();
    let vector_id_docid = env
        .create_poly_database(&mut wtxn, Some(VECTOR_ID_DOCID))
        .unwrap();
    let documents = env
        .create_poly_database(&mut wtxn, Some(DOCUMENTS))
        .unwrap();
    wtxn.commit().unwrap();

    let list = [
        (main, MAIN),
        (word_docids, WORD_DOCIDS),
        (exact_word_docids, EXACT_WORD_DOCIDS),
        (word_prefix_docids, WORD_PREFIX_DOCIDS),
        (exact_word_prefix_docids, EXACT_WORD_PREFIX_DOCIDS),
        (word_pair_proximity_docids, WORD_PAIR_PROXIMITY_DOCIDS),
        (script_language_docids, SCRIPT_LANGUAGE_DOCIDS),
        (
            word_prefix_pair_proximity_docids,
            WORD_PREFIX_PAIR_PROXIMITY_DOCIDS,
        ),
        (
            prefix_word_pair_proximity_docids,
            PREFIX_WORD_PAIR_PROXIMITY_DOCIDS,
        ),
        (word_position_docids, WORD_POSITION_DOCIDS),
        (word_fid_docids, WORD_FIELD_ID_DOCIDS),
        (field_id_word_count_docids, FIELD_ID_WORD_COUNT_DOCIDS),
        (word_prefix_position_docids, WORD_PREFIX_POSITION_DOCIDS),
        (word_prefix_fid_docids, WORD_PREFIX_FIELD_ID_DOCIDS),
        (facet_id_f64_docids, FACET_ID_F64_DOCIDS),
        (facet_id_string_docids, FACET_ID_STRING_DOCIDS),
        (facet_id_exists_docids, FACET_ID_EXISTS_DOCIDS),
        (facet_id_is_null_docids, FACET_ID_IS_NULL_DOCIDS),
        (facet_id_is_empty_docids, FACET_ID_IS_EMPTY_DOCIDS),
        (field_id_docid_facet_f64s, FIELD_ID_DOCID_FACET_F64S),
        (field_id_docid_facet_strings, FIELD_ID_DOCID_FACET_STRINGS),
        (vector_id_docid, VECTOR_ID_DOCID),
        (documents, DOCUMENTS),
    ];

    let rtxn = env.read_txn().unwrap();
    for (db, name) in list {
        let stats = compute_stats(&rtxn, db).unwrap();
        let byte = Byte::from_bytes(stats.size_of_entries);
        let adjusted_byte = byte.get_appropriate_unit(false);

        println!(
            "{name} - {} entries = {}",
            stats.number_of_entries,
            adjusted_byte.to_string()
        );
    }
    Ok(())
}
