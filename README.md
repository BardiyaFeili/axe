# axe

A simple CLI package manager for AppImages, written in Rust.

## Usage

```bash
Usage: axe <COMMAND>

Commands:
  add      Add a package
  list     List all packages in the lockfile
  install  Install all packages defined in the lockfile
  run      Run an installed AppImage by name
  rename   Rename a package in the lockfile
  update   Check for updates for all packages and install them
  remove   Remove a package and its desktop entry
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### Add an application

Works with GitHub shorthand or direct URLs.

```bash
Usage: axe add [OPTIONS] <SOURCE>

Arguments:
  <SOURCE>  Source to add from (GitHub repo 'owner/repo' or a URL)

Options:
      --name <NAME>  Optional override for package name
      --prerelease   Include pre-releases (for GitHub sources)
  -y, --yes          Auto-agree to all prompts
  -d, --desktop      Create a desktop entry for the package
  -h, --help         Print help
```

### Run an app

Axe does have support for .desktop files, but you can also run them like this

```bash
axe run app_name # app_name is not case-sensitive
```

### Update all packages

```bash
axe update
```

### Manage collection

```bash
axe list                          # List installed apps
axe rename <old_name> <new_name>  # Rename a package
axe remove <name>                 # Delete app and desktop entry
axe install                       # Restore apps from lockfile
```

## Config

The lockfile is kept at `~/.config/axe/axe.lock`
This allows you to easily save in your dotfiles repo

## Building from source

First clone the repo and cd into it and then build using cargo

```bash
git clone https://github.com/BardiyaFeili/axe.git && cd axe

cargo build --release
```

Then move the binary to a directory that is in your path.

```bash
mv ./target/release/axe ~/.local/bin
```
