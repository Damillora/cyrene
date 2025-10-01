# cyrene

Conjure upon ripples of past reverie to run multiple versions of runtimes and tools.

## Features

* Install and upgrade multiple versions of tools
* Symlink-based version management
* Extensible with scripts written in [Rune](https://rune-rs.github.io/)
* Synchronize tool versions with `cyrene.toml` lockfiles

## Installation

cyrene should depend only on libc.

A set of plugins are available in the [cyrene-plugins](https://github.com/Damillora/cyrene-plugins) repository.

### From releases

Grab the latest binary from the [Releases](https://github.com/Damillora/cyrene/releases) page.

### Run from source

cyrene is built and tested against latest Rust.

```sh
git clone https://github.com/Damillora/cyrene.git
cd cyrene
cargo install --path cyrene
```

### Managing cyrene with cyrene

To manage `cyrene` with `cyrene` itself, first install `cyrene` version `0.2.3` or later, alongside the [cyrene](https://github.com/Damillora/cyrene-plugins/blob/main/cyrene.rn) plugin.

After that, simply:
```sh
cyrene install cyrene
```

## Usage

[![asciicast](https://asciinema.org/a/2R58mjpKZ40Upx2KdiHdZ1fmx.svg)](https://asciinema.org/a/2R58mjpKZ40Upx2KdiHdZ1fmx)

```sh
# Install multiple versions of runtimes...
cyrene install node@22
cyrene install node@20
# ..and enable one of them at a time.
cyrene link node 20
node
# Welcome to Node.js v20.19.5.
# Type ".help" for more information.
# >
cyrene link node 22
node
# Welcome to Node.js v22.20.0.
# Type ".help" for more information.
# >

# Upgrades are per major version
cyrene upgrade node@22
# Uninstall every Node version
cyrene uninstall node
# Lockfile example
cat << EOF > cyrene.toml
[versions]
node = "20.19.5"
EOF
cyrene load
node
# Welcome to Node.js v20.19.5.
# Type ".help" for more information.
# >
# Load default lockfile
cyrene load -d
```


## Configuration

Cyrene is currently configured with environment variables:
Run `cyrene env` to generate a script exporting its default configuration to `$HOME/.config/cyrene/cyrene.sh`.

* `CYRENE_APPS_DIR`: Location of installed binaries. Defaults to `$HOME/.local/share/cyrene/apps`.
* `CYRENE_PLUGINS_DIR`: Location of installed plugins. Defaults to `$HOME/.local/share/cyrene/apps`.
* `CYRENE_INSTALL_DIR`: Location of the `cyrene` binary itself. Default to the location of the `cyrene` executable itself.

The default lockfile is located at `$HOME/.config/cyrene/cyrene.toml`. Per-project lockfiles are configured using the current directory's `cyrene.toml` file.

## Contributing

cyrene is still in heavy development, but contributions are welcome! Feel free to file an issue or even submit a PR if you want.

## License

cyrene is licensed under the [MIT License](LICENSE).
