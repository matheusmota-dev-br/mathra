# Reading and layout rules

Every rule has a stable id. The code cites the id where the rule is applied (`crates/core/src/rules.rs`), the matching test starts with the same id (`crates/core/tests/rules.rs`), and `mathra --explain` lists the rules that changed the result.

Prefixes: `L` layout and locale, `S` speech and SVG output (see `svg.md`), `D` determinism, `E` recovery from invalid input (see `tolerance.md`). The ids keep their original Portuguese abbreviations (`ESP` spacing, `LET` letters, `ENT` input, `CONTA` arithmetic).

## Numbers

### L-NUM-1 · Decimal comma in the input

In **pt-BR**, a comma directly between two digits is a decimal separator: `3,14` is the number 3.14.

A comma followed by a space is still a list separator: `f(1, 5)` has two arguments.

A dot between digits is decimal in both locales, because that is the LaTeX convention: `3.14` is 3,14 in pt-BR and 3.14 in en.

In **en**, the comma is never decimal.

The number is kept in the tree in canonical form, with a dot: `{"type":"number","value":"3.14"}`.

> A consequence to know about: in pt-BR, `f(1,5)` has **one** argument (1.5). For two, write `f(1, 5)` or `f(1; 5)`.

### L-NUM-2 · How the number is written in the output

The decimal separator drawn comes from the locale, not from the input: pt-BR writes `,`, en writes `.`. The same `3.14` comes out as `3,14` in pt-BR.

### L-NUM-3 · Thousands separator written by the author

In **pt-BR**, a dot followed by groups of exactly three digits separates thousands: `1.000` is one thousand, `12.345,67` is a bit over twelve thousand. In **en**, the comma plays the same role: `1,000.5`. The number is stored without the separator (`"value":"1000"`, `"thousands":"."`) and drawn the way the author wrote it.

> Why: authors write `5,3\times1.000` meaning 5300. Treating the dot as a decimal point wrote `1,000` and changed the meaning. TeX never changes the text; mathra must not change the meaning.

## Intervals

### L-INT-1 · Brazilian open interval

In **pt-BR**, a `]` without an operand before it (at the start, or after a relation, operator or punctuation) opens an interval, and a `[` after an operand, in a span already opened with `]`, closes it. `]2, 5[` gets delimiter spacing, and the speech says "abre intervalo … fecha intervalo".

## Units

### L-UNI-1 · siunitx quantities

`\SI{9,8}{\meter\per\second\squared}` and `\qty{…}{…}` write the number, a thin space and the unit in upright letters. Negative exponents (`\per`) become a slash: `m/s²`, and more than one unit in the denominator gets parentheses: `J/(mol·K)`. `\num{…}` groups with thin spaces from five digits on; `\ang{30}` writes `30°`.

## Input coming from other systems

### L-ENT-1 · HTML entities

`&lt;`, `&gt;`, `&amp;`, `&nbsp;`, `&quot;` inside the equation are read as the characters they stand for. They show up when the equation went through an HTML editor.

### L-ENT-2 · Outer delimiters

`$…$`, `$$…$$`, `\(…\)` and `\[…\]` around the whole equation are removed.

### L-ENT-3 · Text size commands

`\small`, `\scriptsize`, `\large` and similar are ignored: the size comes from whoever uses the SVG.

## Macros

### L-MAC-1 · Substitution macros

`\newcommand{\name}[n][default]{body}`, `\renewcommand`, `\providecommand`, `\def\name#1#2{body}`, `\let\a\b`, `\DeclareMathOperator{\name}{text}` (and `*`), `\DeclarePairedDelimiter{\name}{open}{close}` (with `*` for `\left…\right`) and `\newenvironment{name}{begin}{end}` apply to the rest of the equation. Expansion replaces `#1…#9` with the arguments and reads the result again. After 2,000 expansions mathra stops: the macro almost certainly calls itself (an error in strict mode; in tolerant mode the excess is discarded with a single E-REC-1 warning).

`\ifx`, `\expandafter`, `\catcode` and `\def` with delimited parameters are deliberately not supported (see `boundary.md`).

Package commands defined through other commands go through the same mechanism: `\dv`, `\pdv` (physics), `\ket`, `\bra`, `\braket`, `\Set` (braket), `\abs`, `\norm`, `\sfrac`, `\nicefrac`, `\unitfrac`.

## Column arithmetic (mathra extensions)

### L-CONTA-1 · Long division, Brazilian layout

`\divchave{247}{5}` computes the long division and draws it the way Brazilian schools do: dividend on the left, divisor to the right of the bracket, quotient under the divisor, and each step (product with a minus sign, rule, remainder with the next digit brought down) under the dividend. Steps whose product is zero only bring the digit down.

### L-CONTA-2 · Column arithmetic

`\contaarmada{+}{347}{285}` (also `-` and `\times`) aligns the terms on the right, writes the carries at script size above the first term, draws the rule and writes the computed result. When multiplying by a number with more than one digit, the shifted partial products appear before the sum.

## Functions

### L-FUN-1 · Function name by locale

The input accepts LaTeX's English names and the Brazilian ones (`\sen`, `\tg`, `\cotg`, `\cossec`, `\arcsen`, `\arctg`, `\senh`, `\mdc`, `\mmc`). The tree keeps the canonical English name (`sin`). The output writes it in the locale's language: `sin x` in en, `sen x` in pt-BR.

## Letters

### L-LET-1 · Math italic

Latin letters and lowercase Greek letters are drawn with Unicode's math italic characters (block U+1D400; `h` lives at U+210E). Uppercase Greek letters stay upright, as in TeX.

## Spacing

### L-ESP-1 · Space between atoms

Every node has a TeX atom class (TeXbook, ch. 17): `ord`, `op`, `bin`, `rel`, `open`, `close`, `punct`. The space between neighbors comes from the table in the TeXbook, ch. 18, in *mu* (1/18 em):

| left \ right | ord | op | bin | rel | open | close | punct | inner |
|---|---|---|---|---|---|---|---|---|
| **ord**   | 0 | 3 | (4) | (5) | 0 | 0 | 0 | (3) |
| **op**    | 3 | 3 | – | (5) | 0 | 0 | 0 | (3) |
| **bin**   | (4) | (4) | – | – | (4) | – | – | (4) |
| **rel**   | (5) | (5) | – | 0 | (5) | 0 | 0 | (5) |
| **open**  | 0 | 0 | – | 0 | 0 | 0 | 0 | 0 |
| **close** | 0 | 3 | (4) | (5) | 0 | 0 | 0 | (3) |
| **punct** | (3) | (3) | – | (3) | (3) | (3) | (3) | (3) |
| **inner** | (3) | 3 | (4) | (5) | (3) | 0 | (3) | (3) |

3 = thin space, 4 = medium, 5 = thick. Values in parentheses vanish in script style. `–` cannot happen because of L-ESP-2. Fractions, `\left…\right` and matrices with delimiters are of class `inner`.

### L-ESP-2 · An operator without operands becomes ordinary

A `bin` becomes `ord` when it has no operand on the left (start, or after `bin`, `op`, `rel`, `open`, `punct`) or on the right (end, or before `rel`, `close`, `punct`). That is what keeps the minus of `-b` and of `x = -1` attached (TeXbook, Appendix G, rules 5 and 6).

### Grouping braces

As in TeX, an operator alone in braces becomes `ord`: `{,}` is a comma without punctuation spacing.

## Determinism

### D-DET-1 · Same input, same bytes

The same equation, locale, mathra version and embedded font produce identical output, byte for byte. Nothing depends on the clock, on hash order or on the system.

## Two-dimensional layout

The rules for fractions, roots, scripts, limits, accents, bars, braces and arrows follow Appendix G of the TeXbook with the parameters of the font's MATH table, as MathML Core formulates them. Each lives in `crates/core/src/layout/`:

| Structure | MATH table constants used |
|---|---|
| Fraction (`\frac`, `\dfrac`, `\tfrac`) | `FractionNumerator(DisplayStyle)ShiftUp`, `…DenominatorShiftDown`, `…GapMin`, `FractionRuleThickness`, `AxisHeight`; `\nulldelimiterspace` of 0.12 em on each side |
| Binomial | `StackTopShiftUp`, `StackBottomShiftDown`, `StackGapMin` |
| Root | `RadicalVerticalGap`, `RadicalRuleThickness`, `RadicalExtraAscender`, `RadicalKernBefore/AfterDegree`, `RadicalDegreeBottomRaisePercent` |
| Scripts | `SuperscriptShiftUp(Cramped)`, `SubscriptShiftDown`, `…BaselineDrop…`, `SubSuperscriptGapMin`, `SpaceAfterScript`, italic correction |
| Limits (display `\sum`, `\lim`, `\overset`) | `UpperLimitGapMin`, `UpperLimitBaselineRiseMin`, `LowerLimit…` |
| Large operator | variant with height ≥ `DisplayOperatorMinHeight` in display style, centered on the axis |
| Delimiters (`\left`, matrices) | size variants and assembly from parts; height ≥ max(0.901 × 2δ, 2δ − 0.5 em), like `\delimiterfactor` and `\delimitershortfall` |
| Accents | `AccentBaseHeight`, the glyph's accent attachment; `\widehat` and extended arrows through horizontal assembly |
| Matrices and `array` | strut of 0.84 em / 0.36 em per row, columns 1 em apart, `\jot` of 0.3 em in alignments |

The prime of `f'` uses the ′ glyph at normal size, attached to the base, because it is already drawn as a superscript.

## Measured coverage

`cargo run --release -p mathra --example bench` runs the hand-written constructions in `corpus/` in both modes and prints how many convert; the README has the numbers of one run. The constructions strict mode refuses are not equations: TeX programming (`\ifx`, `\expandafter`, `\catcode`) and diagrams (`tikz-cd`, `xymatrix`, `\tikz`, `\chemfig`). Strict mode also "accepts" LaTeX escaped twice (`\\cdot`), because it is valid for TeX (a line break followed by the letters "cdot"); tolerant mode reads what the author meant and warns (E-REC-2).
