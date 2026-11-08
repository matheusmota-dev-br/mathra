# Tolerant mode

LaTeX copied from other tools is often broken: OCR noise, copies from HTML editors, JSON escapes that were never undone, double scripts. TeX refuses all of it. By default mathra **draws what it can and records a warning** for each problem. With `--strict` (CLI) or `Options { strict: true, .. }` (library), it refuses like TeX.

Warnings show up in `--explain`, on the CLI's stderr, in `Output::warnings` and inside the SVG, as `<mathra:warning rule="…">` in `<metadata>`. In `--batch` mode the line says `warning` instead of `ok`.

| Rule | Problem | What mathra does |
|---|---|---|
| E-REC-1 | Unknown command | If it is `\n` before a known relation (`\nlesssim`), reads it as its negation, `\not\lesssim`. If it starts with a known command (`\vertA`), reads it as `\vert A`. If it is a backslash before a symbol (`\5`), drops the backslash. Otherwise writes the name in red. An unknown chemfig arrow type is drawn as `->`, and TikZ other than `\draw` inside `\chemmove` is ignored. An unknown siunitx unit (`\SI{3}{\furlong}`) is written by its name. A math command or a math-only environment in text (`\text{\mathrm{V}}`, `\text{\begin{pmatrix}…}`, L-TXT-2) is read as math. A macro that never stops expanding is discarded, with a single warning. |
| E-REC-2 | Leftover JSON escape (`\n`, `²`) | `\n` becomes a line break (a space); `\uXXXX` becomes the character. The negations amssymb defines (`\nleq`, `\nsim`, `\nRightarrow`…) are known commands and never read this way. |
| E-REC-3 | Double script (`x_1_2`, `\text{O}^+_{(aq)}` after `_4`); a script mark as the argument of a script (`y__k`, `x^_2`) | The second script attaches to an empty base, as if the author had written `{}`; a mark that is the argument of a script is drawn as that script (`y__k` is y with the subscript `_`, then k). |
| E-REC-4 | Unbalanced braces; in a chemfig structure, an unbalanced branch `(`, bracket or ring | A stray `}` is ignored; an unpaired `{`, `(` or `[` is closed at the end. |
| E-REC-5 | Unpaired `\left`, `\right`, `\begin`, `\end`, `\schemestart` | `\left` without `\right` closes with an empty delimiter; a lone `\right` draws its delimiter; `\end` without `\begin` is ignored. |
| E-REC-6 | Unknown environment | Read as a centered table, with whatever cells and rows it has. |
| E-REC-7 | Missing argument (`\frac{1}`, `\xrightleftharpoons`) | The argument is left empty. |
| E-REC-8 | Character the font lacks | Draws a box (□). |
| E-REC-10 | MathML: unknown element or named entity, element with the wrong number of children (`<mfrac>` with one) | An unknown element's children are read as a row; an unknown entity is dropped; a missing child is left empty. Content MathML is reported as such. Broken XML is an error in both modes: there is no tree to recover. |
| E-REC-9 | Others: unescaped `%` (a comment to the end of the line in TeX, but `15%` pasted into an equation almost always means a percent sign), unpaired `$` in `\text`, a `$` in the middle of the equation (`a + b $ = c`; the outer `$…$` are L-ENT-2), stray `#`, unknown `array` column, invalid delimiter, invalid color, control character (NUL, ESC…), `\divchave`/`\contaarmada` without integers or with a zero divisor, a siunitx value that is not a number (`\SI{x}{\meter}`, `\num{1.2.3}`), `\ang` with more than three parts, a character or bond `\ce` does not know; in chemfig, a stray character, a ring with more bonds than sides (or fewer than 3 or more than 40 sides), a `?` bond never closed, a mechanism arrow to a hook no atom has | The character is drawn as is, or ignored (`%` is drawn as a percent sign, one warning per comment TeX would see); an invalid color is ignored; the calculation is written inline (`a ÷ 0`); the value is drawn as written; the extra ring bonds, the unclosed bond and the arrow are left out, an impossible ring is a hexagon. |

Comments are the one place where the two modes read valid LaTeX differently. In strict mode an unescaped `%` starts a comment, as in TeX: the rest of the line, the line break and the spaces that start the next line are dropped (`p = 15% \cdot q` is `p = 15`). The verbatim arguments of `\verb`, `\url` and `\href` keep their `%`, and `\%` is a percent sign in both modes. Tolerant mode draws the `%` and what follows, with the warning above.

Tolerant mode never changes the meaning of what was correct: every valid equation produces the same tree and the same SVG in both modes (test `t_tol_valid_equation_same_in_both_modes`).

## Limits

Some inputs are refused in both modes, because recovering would make no sense and going on would cost unbounded memory or stack:

| Limit | Value | Why |
|---|---|---|
| Nesting (`{`, `\frac`, `\sqrt`, `\left`, `^{`…) | 64 levels | Fits in a 1 MB stack (wasm, threads). |
| Macro expansions | 2,000, and the text does not grow past a multiple of the input | `\def\a{\a\a}` finishes. |
| `array` columns | 1,000 | `*{50}{*{50}{…}}` does not create millions of columns. |
| Digits in `\divchave`/`\contaarmada` | 30 | The arithmetic is exact in 128-bit integers. |
