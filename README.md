# tuxedo-keyboard-cli

A CLI for interfacing with the [tuxedo keyboard DKMS module](https://github.com/tuxedocomputers/tuxedo-keyboard).

## Requirements
* either `sudo` installed or you must be root in order to use it
* the `tuxedo_keybard` module must be installed and loaded

## How to use

```
$ tuxedo-keyboard-cli <color> --brightness <brightness> --mode <mode>
```

## Development

* `rustup install nightly`
* `cargo run`

## License
GNU General Public License v3.0
