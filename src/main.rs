use clap::{Args, Parser, Subcommand};
use colored::{ColoredString, Colorize};
use indexmap::IndexMap;
use regex::Regex;
use serde_json::Value;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::process::Command;

const MANIFEST_NAME: &str = ".clasp.json";

#[derive(Parser)]
#[command(
    author,
    version,
    about = "Club (CLasp Upstream Bridge) is a CLI tool for managing multiple clasp remotes for the same Google Apps Script project.",
    long_about
)]
#[command(propagate_version = true)]
struct ClubArgs {
    #[command(subcommand)]
    command: ClubCommand,
}

#[derive(Subcommand)]
enum ClubCommand {
    Init(InitCommand),
    List(ListCommand),
    Push(PushCommand),
    Remove(RemoveCommand),
    Rename(RenameCommand),
    Set(SetCommand),
    Login(LoginCommand),
}

#[derive(Args)]
#[clap(
    about = "Initialize club for a clasp project. The .clasp file must already exist in the directory."
)]
struct InitCommand {}

#[derive(Args)]
#[clap(about = "List all remotes and their script IDs.")]
struct ListCommand {}

#[derive(Args)]
#[clap(about = "Remove a remote.")]
struct RemoveCommand {
    #[clap(help = "The name of the remote to remove.")]
    name: String,
}

#[derive(Args)]
#[clap(about = "Rename a remote. If the new name already exists, the command will fail.")]
struct RenameCommand {
    #[clap(help = "The name of the remote to rename.")]
    old_name: String,
    #[clap(help = "The new name for the remote.")]
    new_name: String,
}

#[derive(Args)]
#[clap(about = "Set or create a remote with a given name and ID.")]
struct SetCommand {
    #[clap(help = "The name of the remote to set.")]
    name: String,
    #[clap(help = "The ID of the remote to set.")]
    id: String,
}

#[derive(Args)]
#[clap(about = "Push to a remote. If no remote is specified, defaults to main.")]
struct PushCommand {
    #[clap(help = "The name of the remote to push to.")]
    remote: Option<String>,
    #[clap(short, long, help = "Push to all remotes.")]
    all: bool,
}

#[derive(Args)]
#[clap(about = "Launches the clasp login command.")]
struct LoginCommand {}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct RemoteName(String);

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct RemoteId(String);

#[derive(Debug, Clone)]
struct ClaspConfig {
    root_dir: String,
    script_id: String, // script_id is not a RemoteId because we don't necessarily trust it
    parent_ids: Vec<String>,
    club_remotes: Option<IndexMap<RemoteName, RemoteId>>,
}

#[derive(Debug)]
enum ClubError {
    ManifestNotFound,
    ManifestReadFail(String),
    ManifestWriteFail(String),
    ClubNotSetup,
    ClubAlreadySetup,
    RemoteNotFound,
    RemoteAlreadyExists,
    InvalidRemoteName,
    InvalidRemoteId,
    NoRemotesAvailable,
    BothRemoteAndAllPassed,
    ClaspError(String),
}

impl TryFrom<String> for RemoteId {
    type Error = ClubError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let re = Regex::new(r"[a-zA-Z0-9-_]{57}").unwrap();
        if re.is_match(&value) {
            Ok(RemoteId(value))
        } else {
            Err(ClubError::InvalidRemoteId)
        }
    }
}

impl TryFrom<String> for RemoteName {
    type Error = ClubError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let re = Regex::new(r"[a-zA-Z0-9-_]+").unwrap();
        if re.is_match(&value) {
            Ok(RemoteName(value))
        } else {
            Err(ClubError::InvalidRemoteName)
        }
    }
}

impl Display for ClubError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ClubError::ManifestNotFound => {
                write!(f, "No clasp manifest found. Are you in a clasp project?")
            }
            ClubError::ManifestReadFail(err) => write!(f, "Error reading clasp manifest: {}", err),
            ClubError::ClubNotSetup => write!(
                f,
                "Club is not set up for this project. Run `club init` to set up club."
            ),
            ClubError::RemoteNotFound => write!(f, "Remote not found."),
            ClubError::RemoteAlreadyExists => write!(f, "New remote name already exists. Remove or rename it first."),
            ClubError::InvalidRemoteName => write!(f, "Invalid remote name. Remote names must be alphanumeric and may contain hyphens and underscores."),
            ClubError::InvalidRemoteId => write!(f, "Invalid remote id. Remote IDs are always 57 characters long and contain only alphanumeric characters, hyphens, and underscores."),
            ClubError::ClubAlreadySetup => write!(f, "Club is already set up for this project."),
            ClubError::ManifestWriteFail(err) => write!(f, "Error writing clasp manifest: {}", err),
            ClubError::NoRemotesAvailable => write!(f, "No remotes exist. Run `club set` to add a remote."),
            ClubError::BothRemoteAndAllPassed => write!(f, "Cannot pass both a remote and the --all flag."),
            ClubError::ClaspError(err) => write!(f, "Error running clasp: {}", err),
        }
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

impl TryFrom<Value> for ClaspConfig {
    type Error = ClubError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let root_dir = value["rootDir"]
            .as_str()
            .ok_or(ClubError::ManifestReadFail("rootDir not found".to_string()))?;
        let script_id = value["scriptId"]
            .as_str()
            .ok_or(ClubError::ManifestReadFail(
                "scriptId not found".to_string(),
            ))?;
        let parent_ids: Vec<String> = value["parentId"]
            .as_array()
            .ok_or(ClubError::ManifestReadFail(
                "parentId not found".to_string(),
            ))?
            .iter()
            .map(|id| {
                id.as_str()
                    .ok_or(ClubError::ManifestReadFail(
                        "parentId not found".to_string(),
                    ))
                    .map(|str| str.to_string())
            })
            .collect::<Result<Vec<String>, ClubError>>()?;
        let club_remotes = value["__club__"].as_object().map(|remotes| {
            let mut remote_map = IndexMap::new();
            for (key, value) in remotes {
                remote_map.insert(
                    RemoteName::try_from(key.to_string()).unwrap(),
                    RemoteId::try_from(
                        value
                            .as_str()
                            .expect("Remote ID should be a string")
                            .to_string(),
                    )
                    .unwrap(),
                );
            }
            remote_map
        });
        Ok(ClaspConfig {
            root_dir: root_dir.to_string(),
            script_id: script_id.to_string(),
            parent_ids,
            club_remotes,
        })
    }
}

impl TryFrom<ClaspConfig> for Value {
    type Error = ClubError;

    fn try_from(config: ClaspConfig) -> Result<Self, Self::Error> {
        let mut json = serde_json::json!({
            "rootDir": config.root_dir,
            "scriptId": config.script_id,
            "parentId": config.parent_ids,
        });
        if let Some(remotes) = config.club_remotes {
            let mut remotes_json = serde_json::json!({});
            for (key, value) in remotes {
                remotes_json[key.0] = Value::String(value.0);
            }
            json["__club__"] = remotes_json;
        }
        Ok(json)
    }
}

fn get_project_dir() -> Result<PathBuf, ClubError> {
    Ok(std::env::current_dir().map_err(|e| ClubError::ManifestReadFail(e.to_string()))?)
}

fn get_manifest_path() -> Result<PathBuf, ClubError> {
    Ok(get_project_dir()?.join(MANIFEST_NAME))
}

fn get_clasp_config() -> Result<ClaspConfig, ClubError> {
    let manifest_path = get_manifest_path()?;

    if !manifest_path.exists() {
        return Err(ClubError::ManifestNotFound);
    }
    let manifest_str = std::fs::read_to_string(&manifest_path)
        .map_err(|e| ClubError::ManifestReadFail(e.to_string()))?;
    let manifest_json: Value = serde_json::from_str(&manifest_str)
        .map_err(|e| ClubError::ManifestReadFail(e.to_string()))?;

    Ok(ClaspConfig::try_from(manifest_json)?)
}

fn write_clasp_config(config: ClaspConfig) -> Result<(), ClubError> {
    let manifest_path = get_manifest_path()?;
    let json_str = Value::try_from(config).and_then(|value| {
        serde_json::to_string_pretty(&value)
            .map_err(|e| ClubError::ManifestWriteFail(e.to_string()))
    })?;
    std::fs::write(manifest_path, json_str)
        .map_err(|e| ClubError::ManifestWriteFail(e.to_string()))?;
    Ok(())
}

fn club_list() -> Result<(), ClubError> {
    match get_clasp_config() {
        Err(err) => Err(err),
        Ok(ClaspConfig {
            club_remotes: None, ..
        }) => Err(ClubError::ClubNotSetup),
        Ok(config) => {
            let remotes = config.club_remotes.unwrap();
            for (remote_name, remote_id) in remotes {
                let key_display = {
                    if "main" == remote_name.0 {
                        remote_name.to_string().bold()
                    } else {
                        ColoredString::from(remote_name.to_string())
                    }
                };
                println!("{}: {}", key_display, remote_id);
            }
            Ok(())
        }
    }
}

fn club_set(set_args: SetCommand) -> Result<(), ClubError> {
    let config = get_clasp_config()?;

    let (remote_name, remote_id) = match (
        RemoteName::try_from(set_args.name),
        RemoteId::try_from(set_args.id),
    ) {
        (Ok(remote_name), Ok(remote_id)) => (remote_name, remote_id),
        (Err(err), _) | (_, Err(err)) => {
            return Err(err);
        }
    };

    let mut remotes = config.club_remotes.ok_or(ClubError::ClubNotSetup)?;
    remotes.insert(remote_name, remote_id);

    let new_config = ClaspConfig {
        club_remotes: Some(remotes),
        ..config
    };

    write_clasp_config(new_config)
}

fn club_init() -> Result<(), ClubError> {
    match get_clasp_config() {
        Ok(ClaspConfig {
            club_remotes: Some(_),
            ..
        }) => Err(ClubError::ClubAlreadySetup),
        Err(err) => Err(err),
        Ok(config) => {
            let mut club_remotes = IndexMap::new();

            // If there's a valid script ID in the manifest already, add it as the main remote
            let mut created_main = false;
            if RemoteId::try_from(config.script_id.clone()).is_ok() {
                club_remotes.insert(
                    RemoteName("main".to_string()),
                    RemoteId(config.script_id.clone()),
                );
                created_main = true;
            }

            let new_config = ClaspConfig {
                root_dir: config.root_dir,
                script_id: config.script_id.clone(),
                parent_ids: config.parent_ids,
                club_remotes: Some(club_remotes),
            };
            write_clasp_config(new_config)?;
            if created_main {
                println!(
                    "Club initialized with main remote set to manifest's scriptId: {}",
                    config.script_id
                );
            } else {
                println!("Club initialized an empty configuration.");
            }
            Ok(())
        }
    }
}

fn club_remove(remove_args: RemoveCommand) -> Result<(), ClubError> {
    let config = get_clasp_config()?;

    let remote_name = RemoteName::try_from(remove_args.name)?;
    let mut remotes = config.club_remotes.ok_or(ClubError::ClubNotSetup)?;
    if remotes.shift_remove(&remote_name).is_none() {
        return Err(ClubError::RemoteNotFound);
    }
    let new_config = ClaspConfig {
        club_remotes: Some(remotes),
        ..config
    };

    write_clasp_config(new_config)
}

fn club_rename(rename_args: RenameCommand) -> Result<(), ClubError> {
    let config = get_clasp_config()?;

    let old_name = RemoteName::try_from(rename_args.old_name)?;
    let new_name = RemoteName::try_from(rename_args.new_name)?;
    let mut remotes = config.club_remotes.ok_or(ClubError::ClubNotSetup)?;
    if remotes.contains_key(&new_name) {
        return Err(ClubError::RemoteAlreadyExists);
    }
    let remote_id = remotes
        .shift_remove(&old_name)
        .ok_or(ClubError::RemoteNotFound)?;
    remotes.insert(new_name, remote_id);
    let new_config = ClaspConfig {
        club_remotes: Some(remotes),
        ..config
    };

    write_clasp_config(new_config)
}

fn club_push(push_args: PushCommand) -> Result<(), ClubError> {
    let config = get_clasp_config()?;
    let remotes = config.club_remotes.clone().ok_or(ClubError::ClubNotSetup)?;

    if push_args.remote.is_some() && push_args.all {
        return Err(ClubError::BothRemoteAndAllPassed);
    }
    if remotes.len() == 0 {
        return Err(ClubError::NoRemotesAvailable);
    }

    if push_args.all {
        for (remote_name, remote_id) in remotes {
            push_to_remote(remote_name, remote_id, config.clone())?;
        }
        Ok(())
    } else {
        let remote_name =
            RemoteName::try_from(push_args.remote.unwrap_or_else(|| "main".to_string()))?;
        let remote_id = remotes.get(&remote_name).ok_or(ClubError::RemoteNotFound)?;
        push_to_remote(remote_name, remote_id.clone(), config.clone())
    }
}

fn push_to_remote(
    remote_name: RemoteName,
    remote_id: RemoteId,
    config: ClaspConfig,
) -> Result<(), ClubError> {
    println!("Pushing to {}", remote_name);
    let mut config_copy = config.clone();
    config_copy.script_id = remote_id.0;
    write_clasp_config(config_copy)?;
    let status = Command::new("clasp")
        .arg("push")
        .status()
        .map_err(|e| ClubError::ClaspError(e.to_string()))?;
    let return_val = if status.success() {
        Ok(())
    } else {
        Err(ClubError::ClaspError("clasp push failed".to_string()))
    };
    // Restore the original config
    write_clasp_config(config)?;
    return_val
}

fn club_login() -> Result<(), ClubError> {
    let status = Command::new("clasp")
        .arg("login")
        .status()
        .map_err(|e| ClubError::ClaspError(e.to_string()))?;
    if status.success() {
        Ok(())
    } else {
        Err(ClubError::ClaspError("clasp login failed".to_string()))
    }
}

fn main() {
    let args = ClubArgs::parse();
    if let Err(e) = match args.command {
        ClubCommand::Init(_) => club_init(),
        ClubCommand::List(_) => club_list(),
        ClubCommand::Set(set_args) => club_set(set_args),
        ClubCommand::Remove(remove_args) => club_remove(remove_args),
        ClubCommand::Rename(rename_args) => club_rename(rename_args),
        ClubCommand::Push(push_args) => club_push(push_args),
        ClubCommand::Login(_) => club_login(),
    } {
        println!("{}", e);
    }
}
