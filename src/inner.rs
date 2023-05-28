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
    /// Download and install the new mieli version.
    Upgrade,
    /// Return the current version of mieli.
    Version,
}

impl Inner {
    pub fn execute(self) -> Result<()> {
        match self {
            Inner::Upgrade => upgrade(),
            Inner::AutoComplete { shell } => auto_complete(shell),
            Inner::Version => version(),
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
