# release-utils-rs

[![codecov.io](https://codecov.io/gh/nicholasbishop/release-utils-rs/coverage.svg?branch=main)](https://app.codecov.io/gh/nicholasbishop/release-utils-rs)

This repo contains tools for automatically releasing Rust code. There
are two Rust packages:
* [`auto-release`] - A command-line utility for releasing Rust projects
  via Github Actions.
  * [![Crates.io](https://img.shields.io/crates/v/auto-release)](https://crates.io/crates/auto-release)
* [`release-utils`] - A library providing the building blocks for
  `auto-release`, useful if you need to customize your release beyond
  what `auto-release` provides.
  * [![Crates.io](https://img.shields.io/crates/v/release-utils)](https://crates.io/crates/release-utils) [![Docs.rs](https://docs.rs/release-utils/badge.svg)](https://docs.rs/release-utils)
  
[`auto-release`]: ./auto-release
[`release-utils`]: ./release-utils

## Comparison with other projects

Unlike [cargo-release] and [release-plz], `release-utils-rs` does not
modify any code in the repo. It does not bump versions, update
changelogs, or make any other change to files in the repo. It only
handles releasing to hosts such as crates.io and github.

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
