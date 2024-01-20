# auto-release

[![Crates.io](https://img.shields.io/crates/v/auto-release)](https://crates.io/crates/auto-release) 
[![Docs.rs](https://docs.rs/auto-release/badge.svg)](https://docs.rs/auto-release)

This package provides an executable that can be run from Github Actions
to release code to crates.io and pushing a git tag.

The intended usage is something like this:

1. All code changes needed for a release are made by a developer in a
   regular git commit. The commit includes bumping the version in
   `Cargo.toml`, and any updates to `Cargo.lock`, changelog files, etc.
2. The commit is reviewed and merged through the normal pull request
   process.
3. Once merged, a Github Actions job runs `auto-release`.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE)
or [MIT license](LICENSE-MIT) at your option.

## Contributing

See the [code of conduct] and [contributing.md].

[code of conduct]: ../docs/code-of-conduct.md
[contributing.md]: ../docs/contributing.md

## Disclaimer

This project is not an official Google project. It is not supported by
Google and Google specifically disclaims all warranties as to its quality,
merchantability, or fitness for a particular purpose.
