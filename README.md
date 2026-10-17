<p align="center"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/mathra/main/docs/banner.svg" alt="mathra" width="100%"></p>

# mathra

SVG equations without a TeX install. mathra reads math LaTeX, builds a **tree of meaning** (what each piece is) and draws it with **explicit rules**, numbered and tested. The locale decides the form: English by default, and in pt-BR `3,14` is a single number and `\sin` comes out as `sen`.

```console
$ mathra '3,14' --locale pt-BR --explain
input    3,14
locale   pt-BR
text     3,14
speech   3,14
tree     {"type":"row","children":[{"type":"number","value":"3.14"}]}
rules
  L-NUM-1   '3,14': comma between digits read as decimal separator (3.14)
  L-NUM-2   3.14 written with ',' as decimal separator (pt-BR)

$ mathra 'f(1, 5) = \sin x - 2.5' -o function.svg
```

## Broken LaTeX

By default mathra draws what it can and returns **warnings** for what it had to fix (`E-REC-1` to `E-REC-9`, in `spec/tolerance.md`): unknown commands, unbalanced braces, double scripts, leftover JSON escapes such as `\n` or `\u00b2`. Warnings show up in `--explain`, on stderr, in `Output::warnings` and in the SVG's `<metadata>`. With `--strict`, mathra refuses invalid input instead, like TeX.

```console
$ mathra '\vertA\vert = C_{16,2}' --text
mathra: warning E-REC-1: column 1: unsupported command: \vertA; read as \vert A
|A| = C_(16, 2)
```

## Locales

`en` is the default. `--locale pt-BR` (or `locale="pt-BR"` in Python and JavaScript) switches to Brazilian conventions: `3,14` is one number with a decimal comma, `1.000` is one thousand, `\sin` is written `sen`, `]a, b[` is an open interval, and the spoken phrase is in Portuguese. Every rule that depends on the locale is numbered in `spec/rules.md`.

## What the SVG carries

Besides the outlines, every SVG follows the contract in `spec/svg.md`: speech for screen readers, the source equation and the tree in `<metadata>`, color inherited from the page, reused glyphs, a text layer for selection and unique ids per equation. The same input always produces the same bytes.

## Usage

**Terminal**

```console
$ cargo install mathra-cli
$ mathra 'x \le 2.75; y \ge 1' -o inequality.svg
$ mathra '3,14 + \sin x' --locale pt-BR
$ mathra 'x = -1' --speech        # x equals negative 1
$ echo 'a + b' | mathra --tree
```

**Python** (`crates/python`, with [maturin](https://www.maturin.rs))

```python
import mathra

svg = mathra.tex_to_svg(r"\sin x = 0.5")              # <title>…sine of x equals 0.5</title>
tree = mathra.tex_to_tree("3,14", locale="pt-BR")      # JSON
```

**JavaScript** (`crates/wasm`; the `.wasm` builds with `cargo build -p mathra-wasm --release --target wasm32-unknown-unknown`, packaging with wasm-pack is still to do)

```js
import { texToSvg } from "mathra";
const svg = texToSvg("3.14");
const pt = texToSvg("3,14", "pt-BR");
```

**Rust** (`cargo add mathra`)

```rust
let r = mathra::convert("3.14", &mathra::Options::default())?;
println!("{}", r.svg);
```

## What it converts

Layout follows the parameters of the embedded font's MATH table.

- Fractions (`\frac`, `\dfrac`, `\tfrac`, `\cfrac`, `\over`), binomials, roots with an index
- Scripts, primes (`f'`), limits (`\lim`, `\sum`, `\prod`, `\int` and family, `\limits`)
- `\left … \middle … \right`, `\big` and family, delimiters assembled from parts
- Matrices (`matrix`, `pmatrix`, `bmatrix`, `Bmatrix`, `vmatrix`, `Vmatrix`, `smallmatrix`), `cases`, `aligned`, `align`, `gathered`, `split`, `array` with `|`, `@{}`, `\hline`, `\cline`
- Accents (`\hat`, `\vec`, `\bar`, `\dot`, `\widehat`, `\overrightarrow`…), `\overline`, `\underline`, `\overbrace`/`\underbrace` with labels, `\xrightarrow` and family, `\overset`, `\underset`, `\stackrel`
- `\text` with `$…$` and old-style accents (`\'e`, `\c{c}`), alphabets (`\mathbb`, `\mathcal`, `\mathfrak`, `\mathbf`, `\boldsymbol`, `\mathsf`, `\mathtt`), old `\bf`/`\rm`/`\tt`
- `\cancel`, `\boxed`, `\textcircled`, `\color`, `\phantom`, `\pmod`, `\operatorname`, spaces (`\,` `\quad` `\hspace`)
- siunitx (`\SI`, `\qty`, `\si`, `\num`, `\ang`), basic `\ce`, HTML entities coming from editors
- Substitution macros: `\newcommand`, `\renewcommand`, `\def`, `\let`, `\DeclareMathOperator`, `\DeclarePairedDelimiter`, `\newenvironment`
- Packages: physics (`\dv`, `\pdv`), braket (`\ket`, `\braket`, `\Set`), xfrac, units, `\genfrac`, `\substack`, `\sideset`, `\smash`, `\mathclap`, `\multicolumn`, `\intertext`, `\hdotsfor`, `\raisebox`, `\scalebox`, `\rule`, `\verb`; old syntax `\cases`, `\matrix`, `\eqalign`
- amscd commutative diagrams (`\begin{CD}`)
- mathra extensions: `\divchave{247}{5}` (long division, Brazilian layout) and `\contaarmada{+}{347}{285}` (column arithmetic), both computed

Out of scope, by the rule in `spec/boundary.md` (mathra is for equations): TikZ, `tikz-cd`, `xymatrix`, `\chemfig` and TeX programming (`\ifx`, `\catcode`). Not yet: automatic line breaking, MathML and PNG output, full `\ce`.

## Benchmark

`corpus/` holds 116 hand-written constructions, one or a few per command family and package. One run of `cargo run --release -p mathra --example bench` (arm64, Apple M5, rustc 1.99.1):

| Measure | Value |
|---|---|
| Constructions | 116 |
| Strict: converted | 109 (94.0%) |
| Tolerant: no warning | 109 (94.0%) |
| Tolerant: with warnings | 7 (6.0%) |
| Tolerant: error | 0 (0.0%) |
| Time per equation, mean | 18.7 µs |
| Time per equation, p95 | 46.9 µs |

Times vary by a few microseconds between runs. The 7 that strict mode refuses are not equations: TeX programming (`\ifx`, `\expandafter`, `\catcode`) and diagrams (`tikz-cd`, `xymatrix`, `\tikz`, `\chemfig`). Tolerant mode draws them as text with a warning.

## Layout

```
crates/core      tree, rules, layout, embedded font, outputs (SVG, speech, text)
crates/core/examples  bench (the benchmark above) and banner (the image above)
crates/cli       `mathra` binary
crates/python    Python module (PyO3 + maturin)
crates/wasm      JavaScript module (wasm-bindgen)
spec/            numbered rules (rules.md) and the SVG contract (svg.md)
corpus/          hand-written constructions used by the benchmark
docs/examples/   SVGs generated by mathra itself
```

## Development

```console
$ cargo test                 # one test per rule, with the rule id in its name
$ cargo clippy --all-targets
$ cargo run -p mathra-cli -- '3,14' --locale pt-BR --explain
```

To change behavior, write the rule in `spec/`, cite its id in the code and add a test with the same id.

The tests cover 100% of the lines of the core, the CLI and the Python module. [docs/tests.md](docs/tests.md) explains how to run each suite (including wasm on Node and pytest), what each file checks, the coverage per file and the problem inputs already tested.

The banner above is drawn by mathra itself (`cargo run -p mathra --example banner`); `tests/banner.rs` fails if it goes stale.

## Licenses

Code: your choice of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), as is common in the Rust ecosystem. Embedded font: STIX Two Math, SIL Open Font License 1.1 (`crates/core/fonts/OFL-STIXTwoMath.txt`).
