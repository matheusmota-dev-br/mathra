# Tests

Every case here is an invented equation, written to exercise a construction or a rule.

## How to run

```sh
cargo test --workspace --exclude mathra-python              # core, CLI and wasm (native)
cargo test -p mathra-wasm --target wasm32-unknown-unknown   # wasm on Node (needs wasm-bindgen-cli)
cargo clippy --workspace --all-targets -- -D warnings
```

Python, with a virtual environment that has `maturin` and `pytest`:

```sh
cd crates/python && maturin develop && python -m pytest tests
```

Coverage with [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov), with pytest in the same report:

```sh
eval "$(cargo llvm-cov show-env --export-prefix)"
cargo llvm-cov clean --workspace
cargo test --workspace --exclude mathra-python
(cd crates/python && maturin develop && python -m pytest tests)
cargo llvm-cov report --summary-only        # or --html
```

## Benchmark

`cargo run --release -p mathra --example bench` converts the hand-written constructions in `corpus/` in strict and tolerant mode and prints how many convert and the time per equation. The README shows one run.

## What each file covers

| File | Tests | What it checks |
|---|---:|---|
| `crates/core/tests/rules.rs` | 20 | One test per rule in `spec/rules.md` and `spec/svg.md`; the name starts with the rule id. |
| `crates/core/tests/structures.rs` | 24 | Fractions, roots, matrices, cases, alignments: what sits above, below or beside what. |
| `crates/core/tests/latex_reader.rs` | 80 | Every command and environment the reader knows, every recovery path and, at the end (`p_*`), the problem inputs. |
| `crates/core/tests/layout.rs` | 61 | From the tree to boxes: geometric properties (width, height, alignment, stretching), not exact coordinates. Includes the layout problem cases. |
| `crates/core/tests/outputs.rs` | 9 | Screen reader speech in English and pt-BR, linear text, public shortcuts, error messages, locales. |
| `crates/core/tests/locales.rs` | 5 | The 16 languages: every phrase present, the whole corpus in each one, region codes, speech and number conventions per language. |
| `crates/core/tests/hostile_output.rs` | 5 | Hostile input: the SVG stays valid XML and injects no markup. |
| `crates/core/tests/macros_and_packages.rs` | 11 | Substitution macros (L-MAC-1), package commands and the boundary (`spec/boundary.md`). |
| `crates/core/tests/tolerance.rs` | 11 | Tolerant versus strict mode (`spec/tolerance.md`). |
| `crates/core/tests/banner.rs` | 1 | The README banner (`examples/banner.rs`) is deterministic and matches `docs/banner.svg`. |
| `crates/core/tests/examples.rs` | 2 | The README gallery: every equation is accepted in strict mode, each `docs/examples/*.svg` matches what the code draws, and no stray file sits in that folder. |
| `crates/core/src/font.rs` (unit) | 2 | A glyph without an outline and a quadratic curve, which the embedded font does not have. |
| `crates/cli/tests/cli.rs` | 15 | The binary run as a process: arguments, stdin, `--batch`, writing files, exit codes. |
| `crates/wasm/tests/native.rs` | 2 | Exported functions outside the browser, and the errors before they become a `JsError`. |
| `crates/wasm/tests/web.rs` | 2 | The same functions inside wasm, on Node, including the `JsError`. |
| `npm/test/*.test.*` | 7 | The npm package as published: packed, installed in a scratch project, imported with ESM and `require`; options, `render`, strict errors, locales. Run with `npm/build.sh && npm/test/run.sh`. |
| `crates/python/tests/test_mathra.py` | 7 | The installed Python package: SVG, tree, locale, `ValueError`, version. |

## Coverage

Measured with the procedure above (core, CLI, native wasm and pytest together).

| File | Lines | Regions | Functions |
|---|---:|---:|---:|
| `cli/src/main.rs` | 100% | 99.5% | 100% |
| `core/src/font.rs` | 100% | 99.2% | 100% |
| `core/src/input/latex.rs` | 100% | 97.1% | 100% |
| `core/src/input/symbols.rs` | 100% | 61.5% | 100% |
| `core/src/layout/alphabet.rs` | 100% | 100% | 100% |
| `core/src/layout/arithmetic.rs` | 100% | 96.0% | 100% |
| `core/src/layout/math_box.rs` | 100% | 97.9% | 100% |
| `core/src/layout/mod.rs` | 100% | 97.9% | 100% |
| `core/src/layout/stretch.rs` | 100% | 98.6% | 100% |
| `core/src/layout/table.rs` | 100% | 98.5% | 100% |
| `core/src/lib.rs` | 100% | 99.0% | 100% |
| `core/src/locale.rs` | 100% | 80.2% | 100% |
| `core/src/output/mod.rs` | 100% | 100% | 100% |
| `core/src/output/speech.rs` | 100% | 96.2% | 100% |
| `core/src/output/svg.rs` | 100% | 100% | 100% |
| `core/src/output/text.rs` | 100% | 99.2% | 100% |
| `core/src/rules.rs` | 100% | 100% | 100% |
| `core/src/tree.rs` | 100% | 100% | 100% |
| `python/src/lib.rs` | 100% | 90.4% | 100% |
| `wasm/src/lib.rs` | 85.7% | 88.6% | 71.4% |

About what is missing:

- **Regions below 100%** are mostly arms of symbol tables (`symbols.rs`, `locale.rs`) that share a line with others: the line counts as covered when any arm on it runs. Every command in those tables has at least one test; not every combination of symbol and locale does.
- **`wasm/src/lib.rs`**: the missing lines are the code `#[wasm_bindgen]` generates for the JavaScript boundary. It only runs inside wasm, and `cargo-llvm-cov` does not instrument wasm. That path is tested in `tests/web.rs`, on Node, just without measurement.

## Problem cases

Every row has a test. In no case is there a panic, a hang, a NaN or a negative dimension in the SVG.

### Reader (`latex_reader.rs`, `p_*`)

| Input | Strict | Tolerant |
|---|---|---|
| Unbalanced braces | error | draws, E-REC-4 |
| `\left` without `\right`, `\begin` without `\end` and the other way around | error | draws, E-REC-5 |
| Loose `&` and `\\` outside an environment | implicit alignment | same |
| `\newcommand\a{\a}`, `\def\a{\a\a}` | "too many macros" | discards, a single E-REC-1 warning |
| Exponential expansion (26 levels) | finishes in seconds | same |
| 10,000 levels of `{`, `\frac`, `\sqrt`, `\left`, `^{`, `\not` | nesting error | same; fits in a 1 MB stack |
| 1 MB input | reads, in linear time | same |
| Empty, only spaces, `$$`, `\[\]` | empty equation | same |
| NUL and other control characters | error | ignores, E-REC-9 |
| Emoji, a lone combining accent, Arabic, Hebrew, ZWJ, BOM | reads | reads; missing glyphs become □ (E-REC-8) |
| Half a surrogate pair from JSON (`\ud83d`) | error | no invalid character produced |
| `^^`, `^_^_`, `x^1^2^3` | error | draws, E-REC-3 |
| Unclosed `\sqrt[`, missing argument at the end | error | draws, E-REC-7 |
| `\hdotsfor{10^20}`, `*{50}{*{50}{*{50}{c}}}`, a 10,000-digit number | error or limit | finishes; columns stop at 1,000 |
| `\color{red" onload="x}{a}` | error | draws without color, E-REC-9 |
| `\divchave{a}{0}`, arithmetic with more than 30 digits | error | inline calculation, E-REC-9 |
| `\newcommand\a[1]{#0}` | error | warning (it used to panic) |

### Layout (`layout.rs`)

| Input | Result |
|---|---|
| 1×1 matrix, empty matrix, rows with different column counts | composed by the longest row |
| 200×200 matrix | composed in under 5 s in debug |
| `\left(` around 50 rows | the parenthesis is assembled from font parts; `⟨`, which has no parts, stops at the largest variant |
| `\sqrt` of 60 rows | the rule covers the radicand |
| 30 nested fractions, 20 levels of scripts | the size stops at scriptscript, without shrinking further |
| `\overbrace` over long text | ok |
| `\hline`/`\cline` out of range or reversed, `\multicolumn` wider than the table or `{0}` | no negative rule |
| Hand-built tree with a zero divisor or arithmetic with wrong terms | an error, not a panic |
| `\mathbb{1}`, `\mathbb{\alpha}`, `\mathbb{ж}` | double-struck form when it exists; otherwise the character itself |
| Emoji, U+10FFFF in a script, fraction or root | □ with a warning; in strict mode, `Error::MissingGlyph` |
| Accent, brace or frame over nothing | takes the width of its own drawing |

### Output (`hostile_output.rs`, `outputs.rs`)

| Input | Result |
|---|---|
| `\text{</title><script>…}`, `]]>`, `<!--`, `<?xml` | escaped; no element or `on…` attribute injected |
| NUL, ESC, U+FFFE in the source | become U+FFFD in the metadata (S-SVG-8); they used to produce invalid XML |
| Emoji, RTL, zero-width, CJK | valid XML |
| 40,000 characters of source | valid XML |
| The same equation twice / different equations | same ids / different ids |
| Hand-built tree (impossible `LongDivision`, large operator without limits) | speech and text come out without a panic |

### Command line (`cli.rs`)

| Input | Result |
|---|---|
| `--batch -o` to a missing folder | every line says `error	could not write …` (it used to say `ok` and write nothing) |
| `--batch` with a line that is not UTF-8 | that line says `error`, the following ones go on (the batch used to stop silently) |
| `--batch` with CRLF and empty lines | the `\r` goes; file numbering is kept |
| `-o` pointing to a folder | clear error, exit code 1 |
| Empty equation | empty, valid SVG |
| A 100,000-character argument | converts |
