# cyrene

Conjure upon ripples of past reverie to run multiple versions of runtimes and tools.

## Features

* Install and upgrade multiple versions of tools
* Symlink-based version management
* Extensible with scripts written in Rune
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

```sh
# Install multiple versions of runtimes...
cyrene install node 22
cyrene install node 20
# ..and enable one of them at a time.
cyrene link node 20
# Upgrades are per major version
cyrene upgrade node 22
# Uninstall
cyrene uninstall node
```


## Configuration

## Contributing

cyrene is still in heavy development, but contributions are welcome! Feel free to file an issue or even submit a PR if you want.

## License

cyrene is licensed under the [MIT License](LICENSE).
