# release-utils-rs

[![Crates.io](https://img.shields.io/crates/v/release-utils)](https://crates.io/crates/release-utils) 
[![Docs.rs](https://docs.rs/release-utils/badge.svg)](https://docs.rs/release-utils)

This repo contains a Rust crate with utilities for automatically
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
   blocks for this automated part are what `release-utils-rs` provides.
   
## Comparison with other projects

This is primarily a library crate; the intention is that projects can
incorporate it into an [xtask]-style package that includes a release
command, triggered by a [GitHub Action] or some similar job runner.

Unlike [cargo-release] and [release-plz], this crate does not modify any
code in the repo. It does not bump versions, update changelogs, or make
any other change to files in the repo. It only handles releasing to
hosts such as crates.io and github.

[Github Action]: https://docs.github.com/en/actions
[cargo-release]: https://github.com/crate-ci/cargo-release
[release-plz]: https://github.com/marcoieni/release-plz
[xtask]: https://github.com/matklad/cargo-xtask

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE)
or [MIT license](LICENSE-MIT) at your option.

## Contributing

See the [code of conduct] and [contributing.md].

[code of conduct]: docs/code-of-conduct.md
[contributing.md]: docs/contributing.md

## Disclaimer

This project is not an official Google project. It is not supported by
Google and Google specifically disclaims all warranties as to its quality,
merchantability, or fitness for a particular purpose.
