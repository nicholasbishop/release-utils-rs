# auto-release

[![Crates.io](https://img.shields.io/crates/v/auto-release)](https://crates.io/crates/auto-release)

This package provides an executable that can be run from Github Actions
to release code to crates.io and pushing a git tag.

All code changes needed for a release are made by a developer in a
regular git commit. The commit includes bumping the version in
`Cargo.toml`, and any updates to `Cargo.lock`, changelog files, etc. The
commit is reviewed and merged through the normal pull request
process. Once merged, a Github Actions job runs `auto-release` to
actually push the release.

Releasing this way has a couple advantages over local release flows:
1. It greatly reduces the opportunity for mistakes, such as forgetting
   to sync the local branch or forgetting to push git tags.
2. Cargo credentials can be stored securely on Github, no need for a
   local copy.

## Usage

Create a Github Actions workflow, e.g. `.github/workflows/release.yaml`:

```yaml
on:
  push:
    branches:
      - main

permissions:
  contents: write

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo install auto-release
      - run: auto-release -p <package>
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
```

The `auto-release` crate has minimal compile-time dependencies, so it
compiles very quickly. It does rely on various programs being available,
all of which are already installed and configured on Github's Ubuntu
runners:
* `cargo`
* `curl`
* `gh`
* `git`
* `jq`

### Cargo registry token

Generate the cargo registry token in your crates.io [Account
Settings]. The token scopes must include `publish-update`. If the crate
has never been published before, `publish-new` is also required.

To make the token available to the Github Actions workflow:
1. Go to your repository's settings
2. Click to `Secrets and variables` in the sidebar, then click `Actions`
3. Under `Repository secrets`, click `New repository secret`.

### Options

* `-p`/`--package` can be specified multiple times to release multiple
  packages. Note that order may be significant if packages depend on
  each other.
* `--condition=body` adds a condition that the commit message body must
  start with "release:", otherwise the commit will be ignored.
* `--condition=subject` adds a condition that the commit message subject
  must start with "release:", otherwise the commit will be ignored.

[Account Settings]: https://crates.io/settings/tokens

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
