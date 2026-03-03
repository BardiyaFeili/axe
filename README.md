# axe

A simple CLI package manager for AppImages, written in Rust.

## Usage

### Add an application

Works with GitHub shorthand or direct URLs.

```bash
axe add owner/repo
axe add https://example.com/MyApp.AppImage
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

> [!TIP]
> axe keeps the lockfile at `~/.config/axe/axe.lock`
> This allows you to easily save in your dotfiles repo

The config file is located at `~/.config/axe/axe.toml`

```toml
arch = "x86_64" #auto-generated on first launch
```

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
