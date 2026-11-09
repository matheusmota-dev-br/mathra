# @mathlabs/mathra-editor

Components to build a formula inside a box, drawn by [mathra](../README.md): a cursor, empty places to fill, structures from the keyboard or from buttons. What you edit is what mathra draws, because mathra draws the box itself at every change.

Status: round 1. The field (`<mathra-field>`) and the editor without a screen (`Editor`) work; palettes, a virtual keyboard, selection and reading LaTeX back in are next.

## Use

```html
<script type="module">
  import "@mathlabs/mathra-editor";
</script>

<mathra-field id="f" locale="pt-BR" aria-label="Formula"></mathra-field>
<button onclick="f.insert('frac')">Fraction</button>

<script type="module">
  f.addEventListener("change", (e) => console.log(e.detail.latex, e.detail.speech));
</script>
```

The field loads mathra's WebAssembly itself (`init()` from `@mathlabs/mathra`).

### Typing

| Key | Does |
| --- | --- |
| letters, digits, `+ - * = < > ( ) , ;` | a symbol, with TeX's spacing for its class |
| `/` | a fraction; what is before the cursor, back to an operator, becomes the numerator |
| `^`, `_` | an exponent, an index |
| `\` then a name, then space or Enter | a command: `\pm`, `\alpha`, `\le`, `\sin`…, or a structure: `\frac`, `\sqrt`, `\root`, `\sum`, `\prod`, `\int`, `\lim` |
| arrows, Home, End | move; up and down go between numerator and denominator, exponent and index |
| Backspace | deletes; a structure with something inside is entered from its end |
| Ctrl/Cmd+Z, Shift+Ctrl/Cmd+Z | undo, redo |

### Without a screen

```js
import { Editor } from "@mathlabs/mathra-editor/core";

const e = new Editor();
e.typeText("x=");
e.insert("frac");
e.typeText("1");
e.moveDown();
e.typeText("2");
e.toLatex(); // "x=\\frac{1}{2}"
```

`toLatex({ marked: true })` is what the field draws: each atom and each place marked with `\htmlData{m=<id>}`, so `render(…).boxes` tells where each one sits (`caretRect`, `cursorAt`).

## Customizing

- **CSS variables** on `mathra-field`: `--mathra-ink`, `--mathra-background`, `--mathra-border`, `--mathra-focus`, `--mathra-caret`, `--mathra-slot`, `--mathra-radius`, `--mathra-padding`. The formula's size follows `font-size`.
- **Parts**: `mathra-field::part(box)`, `::part(drawing)`, `::part(caret)`, `::part(slot)` (each empty place), `::part(input)`.
- **Your own interface**: `Editor` and the geometry functions have no DOM.

## How it works

1. The editor keeps a tree of rows and atoms and writes it as LaTeX with marks: `\htmlData{m=n7}{b}`, `\mathbin{\htmlData{m=n8}{-}}` (a marked operator keeps its class through `\mathbin`, `\mathrel`…).
2. mathra draws that LaTeX and returns, with the SVG, the box of every marked group in the SVG's coordinates.
3. The cursor goes at the edge of the atom beside it; a click goes to the innermost place whose box holds it.

Marking moves nothing: the marked LaTeX draws exactly what the plain one draws (tests in `crates/core/tests/boxes.rs` and `test/round1.test.ts`).

## Develop

```sh
npm/build.sh                # from the repository root: mathra's package with `boxes`
cd editor && pnpm install
pnpm test                   # Node 22.18+ runs the TypeScript directly
pnpm build                  # dist/
pnpm demo                   # http://127.0.0.1:4520/editor/demo/
```
