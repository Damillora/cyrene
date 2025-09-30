# cyrene

Conjure upon ripples of past reverie to run multiple versions of runtimes and tools.

## Features

* Install and upgrade multiple versions of tools
* Symlink-based version management
* Extensible with scripts written in [Rune](https://rune-rs.github.io/)
* Synchronize tool versions with `cyrene.toml` lockfiles

## Installation

cyrene should depend only on libc.

### From releases

Grab the latest binary from the [Releases](https://github.com/Damillora/cyrene/releases) page.

### Run from source

cyrene is built and tested against latest Rust.

```sh
git clone https://github.com/Damillora/cyrene.git
cd cyrene
cargo install --path cyrene
```

## Usage

Plugins are installed into `$HOME/.local/share/cyrene/plugins` (configurable with the `CYRENE_PLUGINS_DIR` environment variable).
Apps installed using `cyrene` are located in `$HOME/.local/share/cyrene/apps` (configurable with the `CYRENE_APPS_DIR` environment variable).

```sh
# Install multiple versions of runtimes...
cyrene install node 22
cyrene install node 20
# ..and enable one of them at a time.
cyrene link node 20
# Upgrades are per major version
cyrene upgrade node 22
# Uninstall every Node version
cyrene uninstall node
# Lockfile example
cat << EOF > cyrene.toml
[versions]
node = "20.19.5"
EOF
cyrene load
# Load default lockfile
cyrene load -d
```


## Configuration

Cyrene is currently configured with environment variables:

* `CYRENE_APPS_DIR`: Location of installed binaries
* `CYRENE_PLUGINS_DIR`: Location of installed plugins

The default lockfile is located at `$HOME/.config/cyrene/cyrene.toml`. Per-project lockfiles are configured using the current directory's `cyrene.toml` file.

## Contributing

cyrene is still in heavy development, but contributions are welcome! Feel free to file an issue or even submit a PR if you want.

## License

cyrene is licensed under the [MIT License](LICENSE).
