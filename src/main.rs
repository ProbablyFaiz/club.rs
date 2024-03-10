/* */
use clap::{Parser, Subcommand};

const MANIFEST_NAME: &str = ".clasp.json";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct ClubArgs {
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

#[derive(Parser, Debug)]
struct ListCommand {}




fn club_list() {

}


fn main() {
    let args = ClubArgs::parse();
    match args.command {
        ClubCommand::List(_) => {}
    }
}
