# Club.rs

*Club, but with 100% more Rust!*

Club (CLasp Upstream Bridge) is a small CLI library for making local Google Apps Script development
a bit easier. Specifically, Club provides a way of managing and pushing
to multiple remotes for a single Clasp project without manually editing
`.clasp.json` files.

```
Usage: club <COMMAND>

Commands:
  init    Initialize club for a clasp project. The .clasp file must already exist in the directory.
  list    List all remotes and their script IDs.
  push    Push to a remote. If no remote is specified, defaults to main.
  remove  Remove a remote.
  rename  Rename a remote. If the new name already exists, the command will fail.
  set     Set or create a remote with a given name and ID.
  login   Launches the clasp login command.
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Installation

Club requires Python 3.6 or higher. To install, run:

```bash
git clone https://github.com/ProbablyFaiz/club
cd club
python3 -m pip install .
```

## Usage

At the top level of your project, run `club init` to initialize the project's Club configuration.
If you have a `scriptId` set in your `.clasp.json` file, Club will automatically set that as the
default remote, `main`. Otherwise, you can create any remote you want with `club set <remote> <scriptId>`.

Once you have a remote set, you can push to it with `club push <remote>`. If you don't specify a remote,
Club will push to the default remote, `main`, or the only remote if there is only one. To push to all
remotes simultaneously, use `club push --all`. To push to one or more remotes of your choice, use
`club push <remote1> <remote2> ...`.

To see all usage information and options, run `club <command> --help`.
