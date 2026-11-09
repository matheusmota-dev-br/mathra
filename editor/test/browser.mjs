// The components in a real browser (Chromium, headless): node test/browser.mjs [screenshot folder]
// Serves the repository root itself (npm/pkg and editor/dist must be built), drives demo/index.html
// (the editor filling the page) and test/components.html (the pieces alone), and checks what a person
// would see and do. Exits non-zero if anything fails.

import { chromium } from "playwright-core";
import { createServer } from "node:http";
import { readFile, mkdir } from "node:fs/promises";
import { extname, join } from "node:path";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";

const root = fileURLToPath(new URL("../..", import.meta.url));
const shots = process.argv[2] ?? null;
if (shots) await mkdir(shots, { recursive: true });
const types = { ".html": "text/html; charset=utf-8", ".js": "text/javascript", ".mjs": "text/javascript", ".wasm": "application/wasm", ".svg": "image/svg+xml", ".png": "image/png" };
const server = createServer(async (req, res) => {
  const path = decodeURIComponent(new URL(req.url, "http://x").pathname);
  const file = join(root, path.endsWith("/") ? `${path}index.html` : path);
  try {
    res.writeHead(200, { "content-type": types[extname(file)] ?? "application/octet-stream" }).end(await readFile(file));
  } catch {
    res.writeHead(404).end();
  }
}).listen(0, "127.0.0.1");
await new Promise((r) => server.once("listening", r));
const base = `http://127.0.0.1:${server.address().port}`;

const browser = await chromium.launch();
const results = [];
async function check(name, f) {
  try {
    await f();
    results.push(`ok   ${name}`);
  } catch (e) {
    results.push(`FAIL ${name}\n     ${String(e.message ?? e).split("\n").slice(0, 6).join("\n     ")}`);
  }
}
const mod = process.platform === "darwin" ? "Meta" : "Control";
// colors change with a short transition: pictures and color checks wait for it
const settle = (page) => page.waitForTimeout(350);
const ready = (page) => page.waitForFunction(() => document.getElementById("editor")?.field?.shadowRoot?.querySelector("svg"));

// ---------- the page ----------

const context = await browser.newContext({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 2 });
await context.grantPermissions(["clipboard-read", "clipboard-write"]);
const page = await context.newPage();
const errors = [];
page.on("console", (m) => m.type() === "error" && errors.push(m.text()));
page.on("pageerror", (e) => errors.push(String(e)));
await page.goto(`${base}/editor/demo/`);
await page.evaluate(() => localStorage.clear());
await page.reload();
await ready(page);
const k = page.keyboard;
const ed = page.locator("#editor");
const value = () => page.evaluate(() => document.getElementById("editor").value);
const setValue = (v) => page.evaluate((v) => (document.getElementById("editor").value = v), v);
const focus = () => page.evaluate(() => document.getElementById("editor").focus());
const tab = (name) => ed.locator("mathra-toolbar").locator("[role=tab]").filter({ hasText: name });
const button = (title) => ed.locator("mathra-toolbar").locator(`button[title="${title}"]`);

await check("The editor fills the page: bar, palettes, formula, outputs beside it", async () => {
  const size = await page.evaluate(() => {
    const r = document.getElementById("editor").getBoundingClientRect();
    return [r.width, r.height, innerWidth, innerHeight];
  });
  assert.deepEqual(size.slice(0, 2), size.slice(2));
  const field = await ed.locator("mathra-field").boundingBox();
  const out = await ed.locator("mathra-output").boundingBox();
  assert.ok(out.x > field.x + field.width - 1, "the outputs are beside the formula");
  assert.ok(field.height > 300, `the formula has ${field.height}px of height`);
  assert.equal(await value(), "x=\\frac{-b\\pm\\sqrt{b^{2}-4ac}}{2a}");
  await settle(page);
  if (shots) await page.screenshot({ path: `${shots}/page.png` });
});

await check("The logo and the GitHub link", async () => {
  // the logo loaded (it is an image from the site)
  await page.waitForFunction(() => { const img = document.querySelector('#editor [slot="title"] img'); return img?.complete && img.naturalWidth > 0; }, null, { timeout: 5000 });
  const gh = ed.locator("#github");
  assert.equal(await gh.getAttribute("href"), "https://github.com/matheusmota-dev-br/mathra");
  assert.equal(await gh.getAttribute("target"), "_blank");
  assert.equal(await gh.getAttribute("title"), "Código no GitHub");
  assert.equal(await gh.isVisible(), true);
  if (shots) await page.screenshot({ path: `${shots}/bar.png`, clip: { x: 0, y: 0, width: 1440, height: 60 } });
});

await check("Sixteen palettes, all their tabs in view", async () => {
  const tabs = ed.locator("mathra-toolbar").locator("[role=tab]");
  assert.equal(await tabs.count(), 16);
  for (let i = 0; i < 16; i++) assert.ok(await tabs.nth(i).isVisible(), `tab ${i} hidden`);
});

await check("A Greek letter from its tab; typing goes on in the field", async () => {
  await setValue("");
  await focus();
  await tab("Gregas").click();
  await button("alfa").click();
  await k.type("+1");
  assert.equal(await value(), "\\alpha+1");
});

await check("A structure from a palette takes the selection", async () => {
  await tab("Estruturas").click();
  await k.press(`${mod}+a`);
  await button("Raiz quadrada").click();
  assert.equal(await value(), "\\sqrt{\\alpha+1}");
});

await check("Search finds buttons across palettes; Enter inserts the first", async () => {
  await setValue("");
  const q = ed.locator("mathra-toolbar").locator("input");
  await q.fill("integral dupla");
  const found = await ed.locator("mathra-toolbar").locator(".panel button").count();
  assert.ok(found >= 1, `${found} found`);
  if (shots) await ed.locator("mathra-toolbar").screenshot({ path: `${shots}/search.png` });
  await q.press("Enter");
  assert.match(await value(), /^\\iint_\{\}/);
  await q.fill("zzzz");
  assert.equal(await ed.locator("mathra-toolbar").locator(".empty").isVisible(), true);
  await q.press("Escape");
  assert.equal(await q.inputValue(), "");
});

await check("The command menu opens on \\, filters, and Enter takes the command", async () => {
  await setValue("");
  await focus();
  await k.type("\\fr");
  await page.waitForFunction(() => !document.getElementById("editor").commandMenu.hidden);
  const items = await ed.locator("mathra-command-menu").locator(".item .name").allTextContents();
  assert.equal(items[0], "\\frac");
  await k.press("Enter");
  await k.type("1");
  await k.press("ArrowDown");
  await k.type("2");
  assert.equal(await value(), "\\frac{1}{2}");
});

await check("The outputs follow the field: MathML, speech, braille, PNG", async () => {
  const out = ed.locator("mathra-output");
  await out.locator("[role=tab]").filter({ hasText: "MathML" }).click();
  assert.match(await out.locator("pre").textContent(), /<math[\s\S]*<mfrac>/);
  await out.locator("[role=tab]").filter({ hasText: "Fala" }).click();
  assert.match(await out.locator("pre").textContent(), /fração|meio|sobre/i);
  await out.locator("[role=tab]").filter({ hasText: "Braille" }).click();
  assert.match(await out.locator("pre").textContent(), /[⠀-⣿]/);
  await out.locator("[role=tab]").filter({ hasText: "PNG" }).click();
  await out.locator("img").waitFor();
  assert.ok((await out.locator("img").evaluate((img) => img.naturalWidth)) > 20);
  await out.locator("[role=tab]").filter({ hasText: "LaTeX" }).click();
  assert.equal(await out.locator("pre").textContent(), "\\frac{1}{2}");
});

await check("Copy stays in view beside the tabs, which scroll, and copies", async () => {
  const out = ed.locator("mathra-output");
  const panel = await out.boundingBox();
  const copy = out.locator('[data-act="copy"]');
  const inView = async () => {
    const b = await copy.boundingBox();
    return b.x >= panel.x && b.x + b.width <= panel.x + panel.width + 0.5;
  };
  assert.equal(await inView(), true, "copy in view");
  // the last tab, scrolled to: copy is still in view, and the tabs keep their scroll as the formula changes
  await out.locator("[role=tab]").last().click();
  assert.equal(await inView(), true, "copy in view with the tabs scrolled");
  const scrolled = await out.locator(".tabs").evaluate((t) => t.scrollLeft);
  await focus();
  await k.type("+3");
  assert.equal(await out.locator(".tabs").evaluate((t) => t.scrollLeft), scrolled);
  await out.locator("[role=tab]").first().click();
  await copy.click();
  assert.equal(await page.evaluate(() => navigator.clipboard.readText()), await value());
  await settle(page);
  if (shots) await out.screenshot({ path: `${shots}/outputs.png` });
});

await check("The language menu: native names, keyboard, and everything follows", async () => {
  const lang = ed.locator(".language > button");
  await lang.click();
  const options = ed.locator(".language .menu [role=option]");
  assert.equal(await options.count(), 16);
  assert.equal(await ed.locator('.language .menu [aria-selected="true"]').textContent(), "Português (Brasil)pt-BR");
  await settle(page);
  if (shots) await page.screenshot({ path: `${shots}/language.png`, clip: { x: 840, y: 0, width: 600, height: 620 } });
  await k.press("Escape");
  assert.equal(await ed.locator(".language .menu").count(), 0);
  await lang.click();
  await options.filter({ hasText: "English" }).click();
  await page.waitForFunction(() => document.getElementById("editor").toolbar.shadowRoot.querySelector("[role=tab]")?.textContent === "Structures");
  const out = ed.locator("mathra-output");
  await out.locator("[role=tab]").filter({ hasText: "Speech" }).click();
  assert.match(await out.locator("pre").textContent(), /half|over|fraction/i);
  assert.equal(await page.evaluate(() => localStorage.getItem("mathra-locale")), "en");
  // by keyboard: down twice from English is Español
  await lang.focus();
  await k.press("Enter");
  await k.press("ArrowDown");
  await k.press("ArrowDown");
  await k.press("Enter");
  assert.equal(await page.evaluate(() => document.getElementById("editor").getAttribute("locale")), "es");
  await lang.click();
  await options.filter({ hasText: "Português" }).click();
});

await check("The theme: system, light, dark, remembered", async () => {
  const bg = () => page.evaluate(() => getComputedStyle(document.getElementById("editor").field.shadowRoot.querySelector(".box")).backgroundColor);
  await ed.locator('[data-theme="dark"]').click();
  await settle(page);
  assert.equal(await bg(), "rgb(32, 35, 38)");
  const key = await page.evaluate(() => getComputedStyle(document.getElementById("editor").toolbar.shadowRoot.querySelector(".grid button")).backgroundColor);
  assert.equal(key, "rgb(47, 51, 54)");
  assert.equal(await ed.locator('[data-theme="dark"]').getAttribute("aria-checked"), "true");
  if (shots) await page.screenshot({ path: `${shots}/dark.png` });
  await ed.locator('[data-theme="light"]').click();
  await settle(page);
  assert.equal(await bg(), "rgb(255, 255, 255)");
  assert.equal(await page.evaluate(() => localStorage.getItem("mathra-theme")), "light");
  await page.reload();
  await ready(page);
  assert.equal(await page.evaluate(() => document.getElementById("editor").getAttribute("theme")), "light");
  await ed.locator('[data-theme="auto"]').click();
});

await check("Zoom makes the formula larger and smaller", async () => {
  await setValue("\\frac{1}{2}");
  const size = () => page.evaluate(() => document.getElementById("editor").field.shadowRoot.querySelector("svg").getBoundingClientRect().height);
  const before = await size();
  await ed.locator('[data-act="zoomIn"]').click();
  await ed.locator('[data-act="zoomIn"]').click();
  const after = await size();
  assert.ok(after > before * 1.2, `${before} → ${after}`);
  await ed.locator('[data-act="zoomOut"]').click();
  await ed.locator('[data-act="zoomOut"]').click();
});

await check("Undo, redo and clear in the bar; letters typed in a row undo together", async () => {
  await setValue("");
  await focus();
  await k.type("ab");
  await ed.locator('[data-act="undo"]').click();
  assert.equal(await value(), "");
  await ed.locator('[data-act="redo"]').click();
  assert.equal(await value(), "ab");
  await ed.locator('[data-act="clear"]').click();
  assert.equal(await value(), "");
});

await check("A drag selects, and the selection is drawn", async () => {
  await setValue("a+b+c+d");
  const svg = ed.locator("mathra-field").locator(".drawing svg");
  const b = await svg.boundingBox();
  await page.mouse.move(b.x + 2, b.y + b.height / 2);
  await page.mouse.down();
  await page.mouse.move(b.x + b.width * 0.55, b.y + b.height / 2, { steps: 6 });
  await page.mouse.up();
  const selected = await page.evaluate(() => document.getElementById("editor").editor.selectedLatex());
  assert.ok(selected.startsWith("a+b"), `selected ${selected}`);
  assert.equal(await ed.locator("mathra-field").locator(".selection").isVisible(), true);
});

await check("Copy and paste LaTeX through the clipboard", async () => {
  await focus();
  await k.press(`${mod}+a`);
  await k.press(`${mod}+c`);
  assert.equal(await page.evaluate(() => navigator.clipboard.readText()), "a+b+c+d");
  await page.evaluate(() => navigator.clipboard.writeText("\\sqrt{2}"));
  await k.press("End");
  await k.press(`${mod}+v`);
  assert.equal(await value(), "a+b+c+d\\sqrt{2}");
});

await check("The cursor follows a click", async () => {
  await setValue("xyz");
  const b = await ed.locator("mathra-field").locator(".drawing svg").boundingBox();
  await page.mouse.click(b.x + b.width * 0.36, b.y + b.height / 2);
  await k.type("w");
  assert.equal(await value(), "xwyz");
});

await check("Bhaskara by keyboard, time per change under 16 ms (p95)", async () => {
  await setValue("");
  await page.evaluate(() => (window.mathraTimes.length = 0));
  await focus();
  await k.type("x=/-b\\pm \\sqrt b^2");
  await k.press("ArrowRight");
  await k.type("-4ac");
  await k.press("ArrowDown");
  await k.type("2a");
  assert.equal(await value(), "x=\\frac{-b\\pm\\sqrt{b^{2}-4ac}}{2a}");
  const times = await page.evaluate(() => [...window.mathraTimes].sort((a, b) => a - b));
  const p95 = times[Math.floor(times.length * 0.95)];
  results.push(`     per change in the browser: p50 ${times[Math.floor(times.length / 2)].toFixed(2)} ms, p95 ${p95.toFixed(2)} ms over ${times.length}`);
  assert.ok(p95 < 16, `p95 ${p95}`);
});

await check("A matrix from its tab; & and Enter fill it", async () => {
  await setValue("");
  await focus();
  await tab("Matrizes").click();
  await button("Matriz 2×2").click();
  await k.type("1&0");
  await k.press("Enter");
  await k.type("1");
  await k.press("Shift+Tab");
  await k.type("0");
  assert.equal(await value(), "\\begin{pmatrix}1 & 0 \\\\ 0 & 1\\end{pmatrix}");
  await button("Adicionar linha abaixo").click();
  assert.equal(await page.evaluate(() => document.getElementById("editor").editor.matrixAtCursor()?.matrix.cells.length), 3);
});

await check("The keyboard and the outputs open and close from the bar", async () => {
  await ed.locator('[data-act="keyboard"]').click();
  assert.equal(await ed.locator("mathra-keyboard").isVisible(), true);
  if (shots) {
    await settle(page);
    await page.screenshot({ path: `${shots}/keyboard.png` });
  }
  await ed.locator('[data-act="keyboard"]').click();
  await ed.locator('[data-act="outputs"]').click();
  assert.equal(await ed.locator("mathra-output").isVisible(), false);
  const field = await ed.locator("mathra-field").boundingBox();
  assert.ok(field.width > 1300, `the formula takes the room (${field.width}px)`);
  await ed.locator('[data-act="outputs"]').click();
});

await check("The gallery: every LaTeX example, search, and one opens in the editor", async () => {
  await ed.locator('[data-act="gallery"]').click();
  const gallery = ed.locator("mathra-gallery");
  await gallery.locator(".card").first().waitFor();
  assert.equal(await gallery.locator(".card").count(), 110);
  // every card shows its formula, drawn by mathra
  assert.equal(await gallery.locator(".card .formula svg").count(), 110);
  await settle(page);
  if (shots) await page.screenshot({ path: `${shots}/gallery.png` });
  await gallery.locator("input").fill("Euler");
  assert.ok((await gallery.locator(".card").count()) >= 1);
  await gallery.locator('.card[title="Euler\'s identity"]').click();
  assert.equal(await page.evaluate(() => document.getElementById("editor").shadowRoot.querySelector("dialog.gallery").open), false);
  assert.equal(await value(), "e^{i\\pi}+1=0");
  // it can be undone
  await ed.locator('[data-act="undo"]').click();
  assert.notEqual(await value(), "e^{i\\pi}+1=0");
  await ed.locator('[data-act="redo"]').click();
});

await check("mathra's options: style, braille, strict, animation, PNG size, rules", async () => {
  await ed.locator('[data-act="settings"]').click();
  const s = ed.locator("mathra-settings");
  await s.locator(".settings").waitFor();
  const opts = () => page.evaluate(() => document.getElementById("editor").field.mathraOptions);
  // style
  await s.locator('[data-set="display"][data-value="inline"]').click();
  assert.equal((await opts()).display, false);
  await s.locator('[data-set="display"][data-value="display"]').click();
  assert.equal((await opts()).display, undefined);
  // braille: Nemeth, then the language's (CMU in pt-BR)
  await s.locator('[data-set="braille"][data-value="nemeth"]').click();
  assert.equal(await page.evaluate(() => document.getElementById("editor").field.output.braille_code), "nemeth");
  await s.locator('[data-set="braille"][data-value="auto"]').click();
  assert.equal(await page.evaluate(() => document.getElementById("editor").field.output.braille_code), "cmu");
  // line breaking
  await s.locator("#breaking").check();
  assert.equal((await opts()).maxWidth, 30);
  await s.locator("#breaking").uncheck();
  // PNG size
  await s.locator('[data-set="png"][data-value="2"]').click();
  assert.equal(await ed.locator("mathra-output").getAttribute("png-scale"), "2");
  // the written SVG, with a button to write it again
  await s.locator("#animate").check();
  const out = ed.locator("mathra-output");
  await out.locator("[role=tab]").filter({ hasText: "SVG" }).click();
  assert.match(await out.locator(".image svg").evaluate((svg) => svg.outerHTML), /mr-f|data-step/);
  assert.equal(await out.locator('[data-act="replay"]').isVisible(), true);
  await s.locator("#animate").uncheck();
  // strict mode refuses invalid LaTeX, and says why; the box still shows it
  await s.locator("#strict").check();
  await setValue("\\frac{1}{2} + \\foo");
  assert.equal(await out.locator(".refused").isVisible(), true);
  assert.ok(await page.evaluate(() => document.getElementById("editor").field.shadowRoot.querySelector("svg") !== null));
  await s.locator("#strict").uncheck();
  // the rules mathra applied: the decimal comma of pt-BR
  await setValue("3,14");
  await out.locator("[role=tab]").filter({ hasText: "Regras" }).click();
  assert.match(await out.locator(".rules").textContent(), /L-NUM/);
  await settle(page);
  if (shots) await page.screenshot({ path: `${shots}/settings.png` });
});

await check("A font from the mathfonts catalog (needs the network)", async () => {
  const s = ed.locator("mathra-settings");
  await page.waitForFunction(() => {
    const r = document.getElementById("editor").settings.shadowRoot;
    return r.querySelectorAll(".fonts [data-font]").length > 1 || r.querySelector(".note");
  }, null, { timeout: 15000 });
  const count = await s.locator(".fonts [data-font]").count();
  if (count <= 1) {
    results.push("     (font catalog not reachable: skipped)");
    return;
  }
  await s.locator('[data-font="euler-math"]').click();
  await page.waitForFunction(() => /Euler/.test(document.getElementById("editor").field.output?.svg ?? ""), null, { timeout: 20000 });
  await s.locator('[data-font=""]').click();
  await ed.locator('.settings-panel [data-close]').click();
  assert.equal(await page.evaluate(() => document.getElementById("editor").shadowRoot.querySelector(".settings-panel").hidden), true);
});

const blockPanel = ed.locator("mathra-field").locator(".block");
const blockSource = ed.locator("mathra-field").locator("textarea.src");

await check("Chemistry: \\ce typed opens its LaTeX; the formula follows as it is typed", async () => {
  await setValue("");
  await focus();
  await k.type("\\ce{");
  assert.equal(await blockPanel.isVisible(), true);
  assert.equal(await blockSource.inputValue(), "\\ce{}");
  assert.equal(await blockSource.evaluate((t) => t.selectionStart), 4);
  await k.type("2H2 + O2 -> 2H2O");
  // drawn while typed
  assert.equal(await value(), "\\ce{2H2 + O2 -> 2H2O}");
  assert.match(await blockPanel.locator(".status").textContent(), /sem avisos/);
  await settle(page);
  if (shots) await page.screenshot({ path: `${shots}/block.png` });
  await k.press(`${mod}+Enter`);
  assert.equal(await blockPanel.isVisible(), false);
  // the cursor is after the reaction: typing goes on in math
  await k.type("+x");
  assert.equal(await value(), "\\ce{2H2 + O2 -> 2H2O}+x");
});

await check("Chemistry: a double click on a reaction edits it; the whole edit undoes in one step", async () => {
  await setValue("\\ce{H2O -> H2 + O2}");
  const svg = ed.locator("mathra-field").locator(".drawing svg");
  const b = await svg.boundingBox();
  await page.mouse.move(b.x + b.width / 2, b.y + b.height / 2);
  assert.equal(await ed.locator("mathra-field").locator(".hover").isVisible(), true, "the block is outlined under the pointer");
  await page.mouse.dblclick(b.x + b.width / 2, b.y + b.height / 2);
  assert.equal(await blockPanel.isVisible(), true);
  await blockSource.fill("\\ce{2H2O -> 2H2 + O2}");
  assert.equal(await value(), "\\ce{2H2O -> 2H2 + O2}");
  await blockPanel.locator('[data-block="done"]').click();
  await ed.locator('[data-act="undo"]').click();
  assert.equal(await value(), "\\ce{H2O -> H2 + O2}");
});

await check("Chemistry: Esc takes the edit back; Enter beside a block opens it", async () => {
  await setValue("\\chemfig{A-B}");
  await focus();
  await k.press("End");
  await k.press("Enter");
  assert.equal(await blockPanel.isVisible(), true);
  await blockSource.fill("\\chemfig{A=B}");
  assert.equal(await value(), "\\chemfig{A=B}");
  await k.press("Escape");
  assert.equal(await value(), "\\chemfig{A-B}");
  assert.equal(await blockPanel.isVisible(), false);
});

const dbar = ed.locator("mathra-field").locator(".dbar");

await check("Diagrams by keyboard: a new one, Alt+arrow draws arrows, labels typed in place", async () => {
  await setValue("");
  await focus();
  await tab("Diagramas").click();
  await button("Diagrama novo: digite na célula, Alt+seta cria uma seta").click();
  await k.type("A");
  await k.press("Alt+ArrowRight");
  await k.type("f");
  await k.press("ArrowRight");
  await k.type("B");
  await k.press("Alt+Shift+ArrowDown");
  await k.type("g");
  assert.equal(await value(), '\\begin{tikzcd}A \\arrow[r, "f"] & B \\arrow[d, "g", dashed] \\\\ & {}\\end{tikzcd}');
  // mathra draws all of it: no warning (an empty line reached by an arrow is kept)
  assert.deepEqual(await page.evaluate(() => document.getElementById("editor").field.output.warnings.map((w) => w.description)), []);
  // on an arrow, its bar of styles
  assert.equal(await dbar.locator('[data-run="arrowStyle:dashed"]').getAttribute("aria-pressed"), "true");
  await settle(page);
  if (shots) await page.screenshot({ path: `${shots}/diagram-keys.png` });
});

await check("Diagrams by mouse: a drag from one cell to another draws an arrow; a click on an arrow edits it", async () => {
  // the cells are outlined while the cursor is in the diagram: A, B, (empty), (empty)
  const cells = await page.evaluate(() => {
    const f = document.getElementById("editor").field;
    return [...f.shadowRoot.querySelectorAll(".dgov rect")].map((r) => { const b = r.getBoundingClientRect(); return [b.x + b.width / 2, b.y + b.height / 2]; });
  });
  assert.equal(cells.length, 4, "four cells outlined");
  const [, B, C] = cells;
  // from B down-left to the empty cell
  await page.mouse.move(B[0], B[1]);
  await page.mouse.down();
  await page.mouse.move((B[0] + C[0]) / 2, (B[1] + C[1]) / 2, { steps: 4 });
  await page.mouse.move(C[0], C[1], { steps: 4 });
  await settle(page);
  if (shots) await page.screenshot({ path: `${shots}/diagram-drag.png` });
  await page.mouse.up();
  await k.type("h");
  assert.match(await value(), /B \\arrow\[d, "g", dashed\] \\arrow\[dl, "h"\]/);
  assert.equal(await page.evaluate(() => document.getElementById("editor").editor.selection()), null, "the drag selected nothing");
  assert.deepEqual(await page.evaluate(() => document.getElementById("editor").field.output.warnings.map((w) => w.description)), []);
  // a click on the arrow f: its label, and the bar to restyle it
  const f = await page.evaluate(() => {
    const field = document.getElementById("editor").field;
    const r = [...field.shadowRoot.querySelectorAll(".dgov rect")].map((x) => x.getBoundingClientRect());
    return [(r[0].right + r[1].left) / 2, (r[0].top + r[0].bottom) / 2];
  });
  await page.mouse.click(f[0], f[1]);
  assert.equal(await page.evaluate(() => document.getElementById("editor").editor.arrowAtCursor()?.index), 0);
  await dbar.locator('[data-run="arrowStyle:hook"]').click();
  await dbar.locator('[data-run="arrowBend:left"]').click();
  assert.match(await value(), /A \\arrow\[r, "f", hook, bend left\]/);
  await dbar.locator('[data-run="arrowRemove"]').click();
  assert.doesNotMatch(await value(), /"f"/);
});

await check("Diagrams: the gallery's amscd square, a cell edited, stays amscd", async () => {
  await setValue("\\begin{CD} A @>f>> B \\\\ @VgVV @VVhV \\\\ C @>k>> D \\end{CD}");
  await focus();
  await page.evaluate(() => { const e = document.getElementById("editor").editor; e.setCursor({ row: e.root.children[0].cells[1][1], offset: 1 }); });
  assert.equal(await ed.locator("mathra-field").locator(".dgov rect").count(), 4);
  await k.type("'");
  assert.equal(await value(), "\\begin{CD} A @>f>> B \\\\ @VgVV @VVhV \\\\ C @>k>> D' \\end{CD}");
});

await check("Diagrams: edit as LaTeX, and back to boxes when done", async () => {
  await dbar.locator('[data-run="editAsLatex"]').click();
  assert.equal(await blockPanel.isVisible(), true);
  const src = await blockSource.inputValue();
  await blockSource.fill(src.replace("@>f>>", "@>\\varphi>>"));
  await k.press(`${mod}+Enter`);
  assert.match(await value(), /varphi/);
  assert.equal(await page.evaluate(() => document.getElementById("editor").editor.root.children[0].kind), "diagram");
});

await check("Diagrams: a square from its palette, a label retyped in place", async () => {
  await setValue("");
  await focus();
  await tab("Diagramas").click();
  await button("Quadrado comutativo").click();
  await page.evaluate(() => { const e = document.getElementById("editor").editor; e.selectArrow(e.root.children[0], 0); });
  await k.press("Backspace");
  await k.type("\\varphi ");
  assert.match(await value(), /"\\varphi"/);
  await page.evaluate(() => { const e = document.getElementById("editor").editor; e.setCursor({ row: e.root, offset: 1 }); });
  // the cursor sits after the diagram, at its right edge
  const c = await page.evaluate(() => { const f = document.getElementById("editor").field; return [f.shadowRoot.querySelector(".caret").getBoundingClientRect().x, f.shadowRoot.querySelector(".drawing svg").getBoundingClientRect().right]; });
  assert.ok(Math.abs(c[0] - c[1]) < 6, `caret at ${c[0]}, diagram ends at ${c[1]}`);
  await settle(page);
  if (shots) await page.screenshot({ path: `${shots}/diagram.png` });
});

await check("Diagrams: every gallery diagram and reaction keeps its cursor in the drawing", async () => {
  const bad = await page.evaluate(async () => {
    const { GALLERY } = await import("/editor/dist/gallery.js");
    const ed = document.getElementById("editor");
    const out = [];
    for (const g of GALLERY.filter((g) => ["reactions", "chemistry", "diagrams"].includes(g.id)))
      for (const r of g.rows) {
        ed.value = r.tex;
        await new Promise((res) => requestAnimationFrame(res));
        const f = ed.field;
        const c = f.shadowRoot.querySelector(".caret").getBoundingClientRect();
        const s = f.shadowRoot.querySelector(".drawing svg").getBoundingClientRect();
        if (Math.abs(c.x - s.right) > 6) out.push(r.name || r.caption);
      }
    return out;
  });
  assert.deepEqual(bad, []);
});

await check("No error in the console", async () => {
  assert.deepEqual(errors, []);
});
await context.close();

// ---------- the pieces alone ----------

const alone = await browser.newContext({ viewport: { width: 1000, height: 800 } });
const p3 = await alone.newPage();
await p3.goto(`${base}/editor/test/components.html`);
await p3.waitForFunction(() => document.getElementById("b")?.shadowRoot?.querySelector("svg"));
const v3 = (id) => p3.evaluate((id) => document.getElementById(id).value, id);
await check("Field and command menu alone: the placeholder, then a sum", async () => {
  assert.equal(await p3.locator("#b").locator(".placeholder").isVisible(), true);
  await p3.evaluate(() => document.getElementById("b").focus());
  await p3.keyboard.type("\\sum");
  await p3.waitForFunction(() => !document.querySelector('mathra-command-menu[for="b"]').hidden);
  await p3.keyboard.press("Enter");
  await p3.keyboard.type("k=1");
  await p3.keyboard.press("Tab");
  await p3.keyboard.type("n");
  await p3.keyboard.press("Tab");
  await p3.keyboard.type("k");
  assert.equal(await v3("b"), "\\sum_{k=1}^{n}k");
});
await check("Inline field and keyboard alone", async () => {
  const key = (title) => p3.locator('mathra-keyboard[for="c"]').locator(`button[title="${title}"]`);
  await p3.evaluate(() => document.getElementById("c").focus());
  await key("Fração").click();
  await key("8").click();
  await key("Mover para baixo").click();
  await key("3").click();
  assert.equal(await v3("c"), "\\frac{8}{3}");
  assert.equal(await p3.locator("#c-result").textContent(), "certo");
});
await alone.close();

// ---------- a phone ----------

const phone = await browser.newContext({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 3, isMobile: true, hasTouch: true });
const p2 = await phone.newPage();
await p2.goto(`${base}/editor/demo/`);
await ready(p2);
await check("Phone: the keyboard starts open, the outputs go below, nothing wider than the screen", async () => {
  assert.equal(await p2.evaluate(() => document.getElementById("editor").keyboard.hidden), false);
  const wide = await p2.evaluate(() => document.documentElement.scrollWidth - innerWidth);
  assert.ok(wide <= 0, `${wide}px too wide`);
  const field = await p2.locator("#editor").locator("mathra-field").boundingBox();
  const out = await p2.locator("#editor").locator("mathra-output").boundingBox();
  assert.ok(out.y > field.y, "the outputs are below");
  await settle(p2);
  if (shots) await p2.screenshot({ path: `${shots}/phone.png` });
});
await check("Phone: tapping keys builds a formula", async () => {
  const kb = p2.locator("#editor").locator("mathra-keyboard");
  await p2.evaluate(() => (document.getElementById("editor").value = ""));
  await kb.locator('button[title="Raiz quadrada"]').tap();
  await kb.locator('button[title="2"]').tap();
  assert.equal(await p2.evaluate(() => document.getElementById("editor").value), "\\sqrt{2}");
});
await phone.close();

await browser.close();
server.close();
console.log(results.join("\n"));
process.exit(results.some((r) => r.startsWith("FAIL")) ? 1 : 0);
