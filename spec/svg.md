# The SVG contract

A mathra SVG is an outline drawing that carries the speech, the source and the tree along with it. Every item below is required and has a test in `crates/core/tests/rules.rs`.

```xml
<svg xmlns="http://www.w3.org/2000/svg" xmlns:mathra="urn:mathra:1"
     viewBox="0 -645 1730 658" width="1.73em" height="0.658em"
     style="vertical-align: -0.013em"
     role="img" aria-labelledby="mathra-0f561d1c-speech" lang="en" data-mathra-version="1.4.0">
  <title id="mathra-0f561d1c-speech">3.14</title>
  <metadata>
    <mathra:source type="latex">3.14</mathra:source>
    <mathra:tree>{"type":"row","children":[{"type":"number","value":"3.14"}]}</mathra:tree>
    <mathra:braille code="nemeth">⠼⠒⠨⠂⠲</mathra:braille>
    <mathra:font>STIX Two Math</mathra:font>
  </metadata>
  <style>svg:root{color:#111}</style>
  <defs><path id="mathra-0f561d1c-g19" d="…"/> …</defs>
  <g fill="currentColor" transform="scale(1,-1)"><use href="#mathra-0f561d1c-g19" x="0"/> …</g>
  <text … fill="transparent" aria-hidden="true">3.14</text>
</svg>
```

Warnings from tolerant mode go into the metadata as `<mathra:warning rule="E-REC-1">…</mathra:warning>`.

### S-SVG-1 · Size and baseline

Coordinates in font units, with the baseline at y = 0. `width` and `height` in `em`, to follow the size of the surrounding text. `vertical-align` lowers the SVG by its depth, lining up the equation's baseline with the text's.

### S-SVG-2 · Speech

`role="img"` and `aria-labelledby` pointing to a `<title>` with the phrase in the locale's language.

### S-SVG-3 · Recoverable source

`<metadata>` keeps the original input, with its notation in `<mathra:source type="latex|mathml|asciimath">`, the tree of meaning as JSON and the font name. The namespace is `urn:mathra:1`.

### S-SVG-4 · Color

Glyphs use `fill="currentColor"`.

- **Inline in a page**, the SVG inherits the text color, and the page theme applies.
- **As a standalone document** (`<img>`, an opened file), `svg:root` matches and applies a fixed dark ink (`#111`).

A `<style>` inside an inline SVG applies to the whole page. That is why the only rule is `svg:root`, which does not match when the SVG is inside HTML.

> **Open decision.** Inside `<img>`, the image does not know the page theme. Following `prefers-color-scheme` made the equation almost invisible on a light page with the system in dark mode (Chrome). A `--theme` option may come later.

### S-SVG-5 · Each glyph drawn once

Each glyph becomes a `<path>` in `<defs>`, in order of first appearance, and is reused with `<use>`.

### S-SVG-6 · Text layer

A transparent `<text>`, stretched with `textLength` over the width of the equation, holding the linear text (`3.14 + x`). When the SVG is inline in the page, selection and find-in-page pick up this text. Inside `<img>`, nothing is selectable: that is a browser rule.

### S-SVG-7 · Unique ids per equation

Ids (`…-speech`, `…-g19`) start with a prefix derived from the input, the locale and the version (32-bit FNV-1a). Several equations on one page do not clash, and the same equation always gets the same ids (D-DET-1).

### S-SVG-8 · Always valid XML

All text that comes from outside (source, speech, linear text, warnings, colors) is escaped for `& < > "`. Characters XML 1.0 cannot take even when escaped (control characters from U+0000 to U+001F except tab, LF and CR, plus U+FFFE/U+FFFF) become U+FFFD. Without this, a `\0` pasted into the input made the SVG unreadable for every browser.

### S-SVG-9 · Braille

`<metadata>` also keeps the equation in braille, as Unicode braille cells, with its code: `<mathra:braille code="cmu|nemeth">`. A braille display or an app can read it without converting the equation again. What each code covers is in `braille.md`.
