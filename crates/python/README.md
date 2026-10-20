<p align="center"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/mathra/main/docs/banner.svg" alt="mathra" width="100%"></p>

# mathra

Math LaTeX to accessible SVG, without a TeX install. Every SVG carries the phrase a screen reader says, in 16 languages, and draws with the STIX Two Math font embedded, so it looks the same everywhere. **[Try it in the browser](https://matheusmota-dev-br.github.io/mathra/)**.

```sh
pip install mathra
```

Wheels for Linux, macOS and Windows, for Python 3.9 and later; nothing else to install.

## Usage

```python
import mathra

svg = mathra.tex_to_svg(r"x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}")
open("quadratic.svg", "w").write(svg)

r = mathra.render(r"\frac{3}{4}", locale="fr")
r["speech"]        # 'trois quarts'
r["warnings"]      # [] — broken LaTeX is drawn anyway and reported here (E-REC-*)

mathra.tex_to_svg(r"\sum_{n=1}^{\infty} \frac{1}{n^2}", display=False)   # inline style
mathra.tex_to_svg(r"\vertA\vert", strict=True)                           # ValueError, like TeX
```

| | |
|---|---|
| `tex_to_svg(tex, locale="en", *, display=True, strict=False)` | The SVG as a string. |
| `render(tex, locale="en", *, display=True, strict=False)` | A dict: `svg`, `speech`, `text`, `tree`, `applied`, `warnings`. |
| `tex_to_tree(tex, locale="en", *, display=True, strict=False)` | The tree of meaning as JSON. |
| `locales()` | `[{"code": "en", "name": "English"}, …]`: en, pt-BR, es, fr, de, it, ru, zh, ja, ko, hi, ar, id, tr, nl, pl. |

Errors are `ValueError`. Types are included (`py.typed`).

## What it draws

Every image below is an SVG written by mathra, on a white card so it reads well on dark pages.

<table>
<tr><td align="center"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/mathra/main/docs/examples/quadratic.svg" alt="Quadratic formula"><br><sub>Quadratic formula</sub></td><td align="center"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/mathra/main/docs/examples/gaussian.svg" alt="Gaussian integral"><br><sub>Gaussian integral</sub></td></tr>
<tr><td align="center"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/mathra/main/docs/examples/matrix.svg" alt="m × n matrix with dots"><br><sub>Matrix with dots</sub></td><td align="center"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/mathra/main/docs/examples/piecewise.svg" alt="Piecewise definition"><br><sub>Piecewise definition</sub></td></tr>
<tr><td align="center"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/mathra/main/docs/examples/chemistry.svg" alt="Chemical equation (mhchem)"><br><sub>Chemistry (<code>\ce</code>)</sub></td><td align="center"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/mathra/main/docs/examples/units.svg" alt="Quantity with units (siunitx)"><br><sub>Units (<code>\SI</code>)</sub></td></tr>
<tr><td align="center" colspan="2"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/mathra/main/docs/examples/maxwell.svg" alt="Maxwell's equations"><br><sub>Maxwell's equations</sub></td></tr>
</table>

## What the SVG carries

Speech in `<title>` linked with `aria-labelledby`, the source LaTeX and the tree of meaning in `<metadata>`, color from the page (`currentColor`), size in `em`, a selectable text layer and ids unique to each equation. The same input always gives the same bytes, in Python and in every other package.

## Also for JavaScript, Rust and the terminal

The same core is an npm package (`@mathlabs/mathra`), a Rust crate and a command-line tool: [github.com/matheusmota-dev-br/mathra](https://github.com/matheusmota-dev-br/mathra#install-and-use). Rules, languages and the full list of supported commands are there too.

## License

The code is MIT or Apache-2.0; the embedded STIX Two Math font is under the SIL Open Font License 1.1.
