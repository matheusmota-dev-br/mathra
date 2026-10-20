# mathra

Math LaTeX to accessible SVG, without a TeX install. Every SVG carries the phrase a screen reader says, in 16 languages, and draws with the STIX Two Math font embedded, so it looks the same everywhere. **[Try it in the browser](https://matheusmota-dev-br.github.io/mathra/)**.

```sh
npm install mathra
```

## Node

Works right away:

```js
import { texToSvg, render } from "mathra";      // or: const { texToSvg } = require("mathra")

const svg = texToSvg(String.raw`x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}`);
const { speech } = render(String.raw`\frac{3}{4}`, { locale: "fr" });   // "trois quarts"
```

## Browsers and bundlers (Vite, webpack, esbuild)

Await `init()` once; it fetches `mathra.wasm` next to the module:

```js
import { init, texToSvg } from "mathra";

await init();
document.querySelector("#eq").innerHTML = texToSvg(String.raw`e^{i\pi} + 1 = 0`);
```

Without a bundler, from a CDN:

```html
<script type="module">
  import { init, texToSvg } from "https://cdn.jsdelivr.net/npm/mathra/web/index.js";
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
| `version` | The mathra version inside the package. |

Options: `locale` (`"en"` by default, `"pt-BR"`, `"es"`, `"fr"`, `"de"`, `"it"`, `"ru"`, `"zh"`, `"ja"`, `"ko"`, `"hi"`, `"ar"`, `"id"`, `"tr"`, `"nl"`, `"pl"`), `display` (`true` by default; `false` for inline style) and `strict` (`false` by default: broken LaTeX is drawn with warnings instead of throwing). A plain string as the second argument is read as the locale.

## What the SVG carries

Speech in `<title>` linked with `aria-labelledby`, the source LaTeX and the tree of meaning in `<metadata>`, color from the page (`currentColor`), size in `em`, a selectable text layer and ids unique to each equation. The same input always gives the same bytes.

Rules, languages and the full list of supported commands: [github.com/matheusmota-dev-br/mathra](https://github.com/matheusmota-dev-br/mathra).

## License

The code is MIT or Apache-2.0; the embedded STIX Two Math font is under the SIL Open Font License 1.1.
