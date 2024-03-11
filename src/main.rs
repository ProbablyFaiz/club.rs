/* */
use clap::{Args, Parser, Subcommand};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use colored::{ColoredString, Colorize};

const MANIFEST_NAME: &str = ".clasp.json";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct ClubArgs {
    #[command(subcommand)]
    command: ClubCommand,
}

#[derive(Subcommand)]
enum ClubCommand {
    // Init(InitCommand),
    List(ListCommand),
    // Push(PushCommand),
    // Remove(RemoveCommand),
    // Rename(RenameCommand),
    // Set(SetCommand),
}

#[derive(Args)]
struct ListCommand {}

#[derive(Debug, PartialEq, Eq, Hash)]
struct RemoteName(String);

#[derive(Debug, PartialEq, Eq, Hash)]
struct RemoteId(String);

#[derive(Debug)]
struct ClaspConfig {
    root_dir: String,
    script_id: RemoteId,
    parent_ids: Vec<String>,
    club_remotes: HashMap<RemoteName, RemoteId>,
}

#[derive(Debug)]
enum ClubError {
    ManifestNotFoundError,
    ManifestReadError(String),
}

impl From<std::io::Error> for ClubError {
    fn from(error: std::io::Error) -> Self {
        ClubError::ManifestReadError(error.to_string())
    }
}

impl From<serde_json::Error> for ClubError {
    fn from(error: serde_json::Error) -> Self {
        ClubError::ManifestReadError(error.to_string())
    }
}

impl Display for RemoteName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for RemoteId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn get_project_dir() -> Result<PathBuf, ClubError> {
    Ok(std::env::current_dir().unwrap())
}

fn get_clasp_config() -> Result<ClaspConfig, ClubError> {
    let manifest_path = get_project_dir()?.join(MANIFEST_NAME);

    if !manifest_path.exists() {
        return Err(ClubError::ManifestNotFoundError);
    }
    let manifest_str = std::fs::read_to_string(manifest_path)?;
    let manifest_json: Value = serde_json::from_str(&manifest_str)?;

    let mut club_remotes: HashMap<RemoteName, RemoteId> = HashMap::new();
    for (key, value) in manifest_json["__club__"].as_object().unwrap() {
        club_remotes.insert(
            RemoteName(key.to_string()),
            RemoteId(value.as_str().unwrap().to_string()),
        );
    }
    let parent_ids = manifest_json["parentId"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| id.as_str().unwrap().to_string())
        .collect();

    Ok(ClaspConfig {
        root_dir: manifest_json["rootDir"].as_str().unwrap().to_string(),
        script_id: RemoteId(manifest_json["scriptId"].as_str().unwrap().to_string()),
        parent_ids,
        club_remotes,
    })
}

fn club_list() {
    let config = get_clasp_config().unwrap();
    for (remote_name, remote_id) in config.club_remotes {
        let key_display = {
            if remote_id == config.script_id {
                remote_name.to_string().bold()
            }
            else {
                ColoredString::from(remote_name.to_string())
            }
        };
        println!("{}: {}", key_display, remote_id);
    }
}

fn main() {
    let args = ClubArgs::parse();
    match args.command {
        ClubCommand::List(_) => club_list(),
    }
}
