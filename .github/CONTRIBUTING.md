# Contributing to mathra

Issues and pull requests are welcome. Everyone taking part follows the [Code of Conduct](CODE_OF_CONDUCT.md).

## Reporting a problem

Open an issue with:

- the input, exactly as given (LaTeX, MathML or AsciiMath), and the options (`--locale`, `--strict`, …);
- what mathra produced (the SVG, the warning or the error) and what you expected;
- the version (`mathra --version`, or the crate, npm or PyPI version) and how you called it: CLI, Rust, Python or JavaScript.

`mathra --explain` shows which rules applied; paste its output when the result looks wrong. Security problems go through [the security policy](SECURITY.md), not issues.

## Changing behavior

Behavior is written down before it is coded:

1. Write or change the rule in `spec/`, with its id (for example `L-NUM-1`).
2. Cite the id in the code that implements it.
3. Add a test whose name carries the same id.

For a larger change, open an issue first, so the rule can be agreed on before the code.

Test inputs are made by hand for the test. Do not copy equations from books, exams or other material you do not hold the rights to.

## Before opening a pull request

```console
$ cargo fmt --all
$ cargo clippy --all-targets -- -D warnings
$ cargo test
$ npm/build.sh && npm/test/run.sh     # if the change reaches the npm package
```

The pull request runs the same checks (`.github/workflows/ci.yml`): fmt, clippy, the Rust, wasm, npm and Python tests. [docs/tests.md](../docs/tests.md) explains each suite.

- One change per pull request.
- Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/): `feat(core): …`, `fix(cli): …`, `docs: …`.
- Add what changed for users to `## Unreleased` in [CHANGELOG.md](../CHANGELOG.md) (create the section if it is not there).
- Leave versions alone; they are bumped in release pull requests.
- If the change alters the banner, regenerate it with `cargo run -p mathra --example banner`.

Every pull request needs the approval of the maintainer before merging.

## License

mathra is licensed under either of [MIT](../LICENSE-MIT) or [Apache-2.0](../LICENSE-APACHE), at your option. Unless you explicitly state otherwise, any contribution you intentionally submit for inclusion in mathra, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
