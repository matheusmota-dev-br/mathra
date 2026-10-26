# Boundary: what goes into mathra

mathra renders **equations**. The goal is to convert 100% of equations; what is not an equation goes to another tool. To keep it from growing into a rewritten TeX, every new construction goes through one rule:

> **The core takes what becomes boxes on a line or in a grid, and arrows between the cells of a grid.** Whatever needs free coordinates or running a program stays out.

| In | Out | Why |
|---|---|---|
| Fractions, scripts, roots, delimiters | — | inline typography |
| Matrices, `array`, `cases`, alignments, `\multicolumn` | — | grid |
| Column arithmetic (`\divchave`, `\contaarmada`), factoring with `array` | — | grid of digits |
| Commutative diagrams: amscd (`CD`), `tikz-cd`, `xymatrix` | `tikz-cd`'s TikZ keys beyond arrows (`\tikzcdset`, nodes placed by coordinates, `\draw` inside the diagram) | a grid of cells plus arrows between them: each arrow is fixed by its two cells, its bend and its style, so no coordinate comes from the author |
| Substitution macros: `\newcommand`, `\renewcommand`, `\providecommand`, `\def` with `#1…#9`, `\let`, `\DeclareMathOperator`, `\DeclarePairedDelimiter`, `\newenvironment` without arguments | `\ifx`, `\expandafter`, `\catcode`, `\def` with delimited parameters | replacing text is reading; conditionals and category changes are running a program |
| — | TikZ, pgfplots, `\chemfig`, `circuitikz` | drawing with coordinates; structural chemistry goes to its own library, which uses mathra for the atom labels |

## If figures ever come in

Figures would live in a separate workspace package (for example `crates/figures`), with a meaning-based language of their own: number line, Cartesian plane, Venn diagram, probability tree, labeled geometric figures. It would depend on the core to draw math labels and would use the same SVG contract. The core would never depend on it, and the LaTeX reader would not gain any drawing command.
