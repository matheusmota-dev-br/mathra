# Boundary: what goes into mathra

mathra renders **equations**. The goal is to convert 100% of equations; what is not an equation goes to another tool. To keep it from growing into a rewritten TeX, every new construction goes through one rule:

> **The core takes what becomes boxes on a line or in a grid.** Whatever needs coordinates, curves or running a program stays out.

| In | Out | Why |
|---|---|---|
| Fractions, scripts, roots, delimiters | — | inline typography |
| Matrices, `array`, `cases`, alignments, `\multicolumn` | — | grid |
| Column arithmetic (`\divchave`, `\contaarmada`), factoring with `array` | — | grid of digits |
| amscd commutative diagram (`CD`) | `tikz-cd`, `xymatrix` | `CD` only has straight arrows between neighboring cells; the others have diagonal and curved arrows, which is geometry |
| Substitution macros: `\newcommand`, `\renewcommand`, `\providecommand`, `\def` with `#1…#9`, `\let`, `\DeclareMathOperator`, `\DeclarePairedDelimiter`, `\newenvironment` without arguments | `\ifx`, `\expandafter`, `\catcode`, `\def` with delimited parameters | replacing text is reading; conditionals and category changes are running a program |
| — | TikZ, pgfplots, `\chemfig`, `circuitikz` | drawing with coordinates |

## If figures ever come in

Figures would live in a separate workspace package (for example `crates/figures`), with a meaning-based language of their own: number line, Cartesian plane, Venn diagram, probability tree, labeled geometric figures. It would depend on the core to draw math labels and would use the same SVG contract. The core would never depend on it, and the LaTeX reader would not gain any drawing command.
