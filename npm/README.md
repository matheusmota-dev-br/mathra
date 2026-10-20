<p align="center"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/mathra/main/docs/banner.svg" alt="mathra" width="100%"></p>

# @mathlabs/mathra

Math LaTeX to accessible SVG, without a TeX install. Every SVG carries the phrase a screen reader says, in 16 languages, and draws with the STIX Two Math font embedded, so it looks the same everywhere. **[Try it in the browser](https://matheusmota-dev-br.github.io/mathra/)**.

```sh
npm install @mathlabs/mathra
```

## What it draws

Every image below is an SVG written by mathra, on a white card so it reads well on dark pages.

<table>
<tr><td align="center"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/mathra/main/docs/examples/quadratic.svg" alt="Quadratic formula"><br><sub>Quadratic formula</sub></td><td align="center"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/mathra/main/docs/examples/gaussian.svg" alt="Gaussian integral"><br><sub>Gaussian integral</sub></td></tr>
<tr><td align="center"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/mathra/main/docs/examples/matrix.svg" alt="m × n matrix with dots"><br><sub>Matrix with dots</sub></td><td align="center"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/mathra/main/docs/examples/piecewise.svg" alt="Piecewise definition"><br><sub>Piecewise definition</sub></td></tr>
<tr><td align="center"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/mathra/main/docs/examples/chemistry.svg" alt="Chemical equation (mhchem)"><br><sub>Chemistry (<code>\ce</code>)</sub></td><td align="center"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/mathra/main/docs/examples/units.svg" alt="Quantity with units (siunitx)"><br><sub>Units (<code>\SI</code>)</sub></td></tr>
<tr><td align="center" colspan="2"><img src="https://raw.githubusercontent.com/matheusmota-dev-br/mathra/main/docs/examples/maxwell.svg" alt="Maxwell's equations"><br><sub>Maxwell's equations</sub></td></tr>
</table>

And what a screen reader hears, from the SVG's `<title>`:

| Equation | `locale` | Speech |
|---|---|---|
| `x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}` | `en` | x equals fraction, negative b plus or minus square root of b squared minus 4 a c, end root, over 2 a, end fraction |
| `\frac{3}{4} + \sqrt{x}` | `fr` | trois quarts plus racine carrée de x |
| `\frac{3}{4} + \sqrt{x}` | `ja` | 4 分の 3 プラス ルート x |
| `\sin^2 \theta + \cos^2 \theta = 1` | `pt-BR` | seno ao quadrado de teta mais cosseno ao quadrado de teta é igual a 1 |

## Node

Works right away:

```js
import { texToSvg, render } from "@mathlabs/mathra";      // or: const { texToSvg } = require("@mathlabs/mathra")

const svg = texToSvg(String.raw`x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}`);
const { speech } = render(String.raw`\frac{3}{4}`, { locale: "fr" });   // "trois quarts"
```

## Browsers and bundlers (Vite, webpack, esbuild)

Await `init()` once; it fetches `mathra.wasm` next to the module:

```js
import { init, texToSvg } from "@mathlabs/mathra";

await init();
document.querySelector("#eq").innerHTML = texToSvg(String.raw`e^{i\pi} + 1 = 0`);
```

Without a bundler, from a CDN:

```html
<script type="module">
  import { init, texToSvg } from "https://cdn.jsdelivr.net/npm/@mathlabs/mathra/web/index.js";
  await init();
  document.body.innerHTML = texToSvg("3,14", { locale: "pt-BR" });
</script>
```

## API

| | |
|---|---|
| `init(input?)` | Loads the wasm. Needed once in browsers and bundlers; does nothing in Node. `input` overrides where the wasm comes from (a URL, a `Response`, bytes or a `WebAssembly.Module`). |
| `texToSvg(tex, options?)` | The SVG as a string. |
| `render(tex, options?)` | `{ svg, speech, text, tree, applied, warnings }`. |
| `locales()` | `[{ code: "en", name: "English" }, …]`. |
| `version` | The version of mathra's core inside the package (the npm package has its own version). |

Options: `locale` (`"en"` by default, `"pt-BR"`, `"es"`, `"fr"`, `"de"`, `"it"`, `"ru"`, `"zh"`, `"ja"`, `"ko"`, `"hi"`, `"ar"`, `"id"`, `"tr"`, `"nl"`, `"pl"`), `display` (`true` by default; `false` for inline style) and `strict` (`false` by default: broken LaTeX is drawn with warnings instead of throwing). A plain string as the second argument is read as the locale.

## What the SVG carries

Speech in `<title>` linked with `aria-labelledby`, the source LaTeX and the tree of meaning in `<metadata>`, color from the page (`currentColor`), size in `em`, a selectable text layer and ids unique to each equation. The same input always gives the same bytes.

## Also for Python, Rust and the terminal

The same core is a Python module, a Rust crate and a command-line tool, and every one gives the same SVG, byte for byte. How to install each: [github.com/matheusmota-dev-br/mathra](https://github.com/matheusmota-dev-br/mathra#install-and-use). Rules, languages and the full list of supported commands are there too.

## License

The code is MIT or Apache-2.0; the embedded STIX Two Math font is under the SIL Open Font License 1.1.
