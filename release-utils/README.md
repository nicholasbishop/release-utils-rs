# release-utils

[![Crates.io](https://img.shields.io/crates/v/release-utils)](https://crates.io/crates/release-utils)
[![Docs.rs](https://docs.rs/release-utils/badge.svg)](https://docs.rs/release-utils)

This package contains a library with utilities for automatically
releasing Rust code.

The intended usage is something like this (but not necessarily exactly
this):

1. All code changes needed for a release are made by a developer in a
   regular git commit. The commit includes bumping the version in
   `Cargo.toml`, and any updates to `Cargo.lock`, changelog files, etc.
2. The commit message is prefixed with `release:` to mark the commit as
   a release trigger.
3. The commit is reviewed and merged through the normal pull request
   process.
4. Once merged, an automatic job sees the specially-marked commit and
   triggers any actions necessary to push the release. The building
   blocks for this automated part are what `release-utils` provides.

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
