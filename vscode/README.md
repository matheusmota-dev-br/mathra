# mathra for VS Code

Math in Markdown and LaTeX drawn by [mathra](https://github.com/matheusmota-dev-br/mathra#readme): accessible SVG, chemistry and commutative diagrams included, with no TeX install.

- **Hover over an equation** in a Markdown or LaTeX file to see it drawn over its LaTeX (the hover's copy button copies the LaTeX), then the phrase a screen reader says, what mathra worked around (an unclosed brace, an unknown command), a link to the visual editor and links to copy it in each form.
- **The Markdown preview's math, drawn by mathra:** `$…$` and `\(…\)` inline; `$$…$$`, `\[…\]`, ```` ```math ```` blocks and LaTeX's display environments (`equation`, `align`, `gather`…) as blocks. Each SVG carries its speech, so a screen reader reads the equation.
- **A visual editor** (**Open Visual Editor**, Cmd/Ctrl+Alt+M, or the editor's context menu): mathra's editor beside the document, opened on the equation at the cursor or the selection. Build the formula with palettes, a keyboard and every structure, chemistry and diagrams included; **Insert in the document** (or Cmd/Ctrl+S) writes it back over the equation, or at the cursor between `$`.
- **Copy Equation As…** SVG, MathML, speech, braille or linear text; **Save Equation as SVG…** or **as PNG…** (three times the size, drawn by resvg); **Explain Equation**, what `mathra --explain` writes: the tree, the speech, braille and every rule that changed the result. Each acts on the equation under the cursor, or the selection.

Everything runs inside VS Code, on mathra's WebAssembly: nothing is sent anywhere.

## Settings

| Setting | Default | |
|---|---|---|
| `mathra.locale` | `auto` | How equations are read, written and spoken, in 16 languages: in `pt-BR` `3,14` is one number and `\sin` is `sen`. `auto` follows the language of VS Code. |
| `mathra.hover.enabled` | `true` | The hover over equations. |
| `mathra.preview.enabled` | `true` | mathra in the Markdown preview. |

The extension turns VS Code's built-in preview math (`markdown.math.enabled`) off by default, so the two do not compete for the same `$…$`; turning it back on leaves the preview's math to whichever plugin reads it first.

## Developing

From the repository, with `npm/pkg` and the editor built (`npm/build.sh`, then `pnpm build` in `editor/`):

```sh
cd vscode
pnpm install
pnpm test        # the tests; `pnpm coverage` asks every line, branch and function of src/ to run
pnpm typecheck
pnpm package     # mathra-<version>.vsix, to install with `code --install-extension`
```

To try it while changing it, open the repository in VS Code and press **F5** (*mathra: run the VS Code extension*): a second window opens on `vscode/demo/` with the extension loaded from source, and a watch task rebuilds it on each save; **Cmd+R** in that window reloads it.

`src/extension.ts` and `src/panel.ts` (the visual editor's webview) are the files that know VS Code; `webview/` is the editor's page, run in the webview; `find.ts` (where the math of a document is), `draw.ts` (mathra's outputs as hovers and HTML) and `markdown.ts` (the preview's markdown-it plugin) are plain TypeScript. The tests run them in Node, `extension.ts` against a stand-in for the `vscode` module (`test/vscode.ts`).
