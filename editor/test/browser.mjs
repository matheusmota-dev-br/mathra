// The components in a real browser (Chromium, headless): node test/browser.mjs [screenshot folder]
// Serves the repository root itself (npm/pkg and editor/dist must be built), drives demo/index.html
// and checks what a person would see and do. Exits non-zero on the first failure.

import { chromium } from "playwright-core";
import { createServer } from "node:http";
import { readFile, mkdir } from "node:fs/promises";
import { extname, join } from "node:path";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";

const root = fileURLToPath(new URL("../..", import.meta.url));
const shots = process.argv[2] ?? null;
if (shots) await mkdir(shots, { recursive: true });
const types = { ".html": "text/html; charset=utf-8", ".js": "text/javascript", ".mjs": "text/javascript", ".wasm": "application/wasm" };
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
const url = `http://127.0.0.1:${server.address().port}/editor/demo/`;

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

const context = await browser.newContext({ viewport: { width: 1000, height: 900 }, deviceScaleFactor: 2 });
const page = await context.newPage();
const errors = [];
page.on("console", (m) => m.type() === "error" && errors.push(m.text()));
page.on("pageerror", (e) => errors.push(String(e)));
await page.goto(url);
await page.waitForFunction(() => document.getElementById("a")?.shadowRoot?.querySelector("mathra-field")?.shadowRoot?.querySelector("svg"));
const k = page.keyboard;
const mod = process.platform === "darwin" ? "Meta" : "Control";
const latexOf = (id) => page.evaluate((id) => document.getElementById(id).value, id);
// focuses a field, by its id or inside the editor
const focus = (id) => page.evaluate((id) => { const el = document.getElementById(id); (el.field ?? el).focus(); }, id);

await check("A starts with its value, drawn", async () => {
  assert.equal(await latexOf("a"), "x=\\frac{-b\\pm\\sqrt{b^{2}-4ac}}{2a}");
});

await check("A: ten palettes in tabs; a Greek letter from its tab", async () => {
  const tabs = page.locator("#a").locator("mathra-toolbar").locator("[role=tab]");
  assert.equal(await tabs.count(), 10);
  await focus("a");
  await page.evaluate(() => document.getElementById("a").editor.selectAll());
  await k.press("Backspace");
  await tabs.filter({ hasText: "Gregas" }).click();
  await page.locator("#a").locator("mathra-toolbar").locator('button[title="alfa"]').click();
  assert.equal(await latexOf("a"), "\\alpha");
});

await check("A: palette buttons keep the focus in the field, typing goes on", async () => {
  await k.type("+1");
  assert.equal(await latexOf("a"), "\\alpha+1");
});

await check("A: a structure from the palette takes the selection", async () => {
  await page.locator("#a").locator("mathra-toolbar").locator("[role=tab]").filter({ hasText: "Estruturas" }).click();
  await k.press(`${mod}+a`);
  await page.locator("#a").locator("mathra-toolbar").locator('button[title="Raiz quadrada"]').click();
  assert.equal(await latexOf("a"), "\\sqrt{\\alpha+1}");
});

await check("A: the command menu opens on \\, filters, and Enter takes the command", async () => {
  await k.press(`${mod}+a`);
  await k.press("Backspace");
  await k.type("\\fr");
  const menu = page.locator("#a").locator("mathra-command-menu");
  await page.waitForFunction(() => !document.getElementById("a").commandMenu.hidden);
  const items = await menu.locator(".item .name").allTextContents();
  assert.ok(items[0] === "\\frac", `first item ${items[0]}`);
  if (shots) await page.screenshot({ path: `${shots}/menu.png` });
  await k.press("Enter");
  await k.type("1");
  await k.press("ArrowDown");
  await k.type("2");
  assert.equal(await latexOf("a"), "\\frac{1}{2}");
  assert.equal(await page.evaluate(() => document.getElementById("a").commandMenu.hidden), true);
});

await check("A: the outputs follow the field: MathML, speech, braille, PNG", async () => {
  const out = page.locator("#a").locator("mathra-output");
  await out.locator("[role=tab]").filter({ hasText: "MathML" }).click();
  assert.match(await out.locator("pre").textContent(), /<math[\s\S]*<mfrac>/);
  await out.locator("[role=tab]").filter({ hasText: "Fala" }).click();
  assert.match(await out.locator("pre").textContent(), /fração|meio|1 sobre 2|um meio/i);
  await out.locator("[role=tab]").filter({ hasText: "Braille" }).click();
  assert.match(await out.locator("pre").textContent(), /[⠀-⣿]/);
  await out.locator("[role=tab]").filter({ hasText: "PNG" }).click();
  await out.locator("img").waitFor();
  const w = await out.locator("img").evaluate((img) => img.naturalWidth);
  assert.ok(w > 20, `PNG ${w}px wide`);
  await out.locator("[role=tab]").filter({ hasText: "LaTeX" }).click();
  assert.equal(await out.locator("pre").textContent(), "\\frac{1}{2}");
});

await check("A: undo, redo and clear buttons", async () => {
  const act = (name) => page.locator("#a").locator(`.actions button[title="${name}"]`);
  await act("Desfazer").click();
  assert.equal(await latexOf("a"), "\\frac{1}{}");
  await act("Refazer").click();
  assert.equal(await latexOf("a"), "\\frac{1}{2}");
  await act("Limpar").click();
  assert.equal(await latexOf("a"), "");
  await act("Desfazer").click();
  assert.equal(await latexOf("a"), "\\frac{1}{2}");
});

await check("A: a drag selects, and the selection is drawn", async () => {
  await page.evaluate(() => (document.getElementById("a").value = "a+b+c+d"));
  const svg = page.locator("#a").locator("mathra-field").locator(".drawing svg");
  const b = await svg.boundingBox();
  await page.mouse.move(b.x + 2, b.y + b.height / 2);
  await page.mouse.down();
  await page.mouse.move(b.x + b.width * 0.55, b.y + b.height / 2, { steps: 6 });
  await page.mouse.up();
  const selected = await page.evaluate(() => document.getElementById("a").editor.selectedLatex());
  assert.ok(selected.startsWith("a+b"), `selected ${selected}`);
  const sel = page.locator("#a").locator("mathra-field").locator(".selection");
  assert.equal(await sel.isVisible(), true);
});

await check("A: copy and paste LaTeX through the clipboard", async () => {
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  await focus("a");
  await k.press(`${mod}+a`);
  await k.press(`${mod}+c`);
  const copied = await page.evaluate(() => navigator.clipboard.readText());
  assert.equal(copied, "a+b+c+d");
  await page.evaluate(() => navigator.clipboard.writeText("\\sqrt{2}"));
  await k.press("End");
  await k.press(`${mod}+v`);
  assert.equal(await latexOf("a"), "a+b+c+d\\sqrt{2}");
});

await check("A: the cursor follows a click", async () => {
  await page.evaluate(() => (document.getElementById("a").value = "xyz"));
  const svg = page.locator("#a").locator("mathra-field").locator(".drawing svg");
  const b = await svg.boundingBox();
  await page.mouse.click(b.x + b.width * 0.36, b.y + b.height / 2);
  await k.type("w");
  assert.equal(await latexOf("a"), "xwyz");
});

await check("A: Bhaskara by keyboard, time per key under 16 ms (p95)", async () => {
  await page.evaluate(() => { document.getElementById("a").value = ""; window.mathraTimes.length = 0; });
  await focus("a");
  await k.type("x=/-b\\pm \\sqrt b^2");
  await k.press("ArrowRight");
  await k.type("-4ac");
  await k.press("ArrowDown");
  await k.type("2a");
  assert.equal(await latexOf("a"), "x=\\frac{-b\\pm\\sqrt{b^{2}-4ac}}{2a}");
  const times = await page.evaluate(() => [...window.mathraTimes].sort((a, b) => a - b));
  const p95 = times[Math.floor(times.length * 0.95)];
  results.push(`     per change in the browser: p50 ${times[Math.floor(times.length / 2)].toFixed(2)} ms, p95 ${p95.toFixed(2)} ms over ${times.length}`);
  assert.ok(p95 < 16, `p95 ${p95}`);
  if (shots) await page.screenshot({ path: `${shots}/a.png`, fullPage: false });
});

await check("A: a matrix from its tab; & and Enter fill it", async () => {
  await page.evaluate(() => (document.getElementById("a").value = ""));
  await page.locator("#a").locator("mathra-toolbar").locator("[role=tab]").filter({ hasText: "Matrizes" }).click();
  await page.locator("#a").locator("mathra-toolbar").locator('button[title="Matriz entre colchetes"]').click();
  await k.type("1&0");
  await k.press("Enter");
  await k.type("1");
  await k.press("Shift+Tab");
  await k.type("0");
  assert.equal(await latexOf("a"), "\\begin{bmatrix}1 & 0 \\\\ 0 & 1\\end{bmatrix}");
  await page.locator("#a").locator("mathra-toolbar").locator('button[title="Adicionar linha abaixo"]').click();
  assert.equal(await page.evaluate(() => document.getElementById("a").editor.matrixAtCursor()?.matrix.cells.length), 3);
});

await check("B: the placeholder, then a sum from the command menu", async () => {
  assert.equal(await page.locator("#b").locator(".placeholder").isVisible(), true);
  await focus("b");
  await k.type("\\sum");
  await page.waitForFunction(() => !document.querySelector('mathra-command-menu[for="b"]').hidden);
  await k.press("Enter");
  await k.type("k=1");
  await k.press("Tab");
  await k.type("n");
  await k.press("Tab");
  await k.type("k");
  assert.equal(await latexOf("b"), "\\sum_{k=1}^{n}k");
  if (shots) await page.locator("#b").screenshot({ path: `${shots}/b.png` });
});

await check("C: the keyboard on the screen answers the question", async () => {
  const key = (title) => page.locator('mathra-keyboard[for="c"]').locator(`button[title="${title}"]`);
  await focus("c");
  await key("Fração").click();
  await key("8").click();
  await key("Mover para baixo").click();
  await key("3").click();
  assert.equal(await latexOf("c"), "\\frac{8}{3}");
  await key("Mover para a direita").click();
  assert.equal(await page.locator("#c-result").textContent(), "certo");
  await page.locator('mathra-keyboard[for="c"]').locator('button[title="Letras"]').click();
  await key("Maiúsculas").click();
  await key("q").click();
  assert.equal(await latexOf("c"), "\\frac{8}{3}Q");
  if (shots) await page.locator(".quiz").screenshot({ path: `${shots}/c.png` });
});

await check("Themes: a variable set outside reaches the field inside the editor", async () => {
  await page.selectOption("#theme", "lousa");
  const bg = await page.evaluate(() => getComputedStyle(document.getElementById("a").field.shadowRoot.querySelector(".box")).backgroundColor);
  assert.equal(bg, "rgb(35, 57, 47)");
  // the colors change with a short transition: the picture waits for it
  await page.waitForTimeout(400);
  if (shots) await page.screenshot({ path: `${shots}/lousa.png` });
  await page.selectOption("#theme", "dark");
  const dark = await page.evaluate(() => getComputedStyle(document.getElementById("b").shadowRoot.querySelector(".box")).backgroundColor);
  assert.equal(dark, "rgb(32, 35, 38)");
  await page.waitForTimeout(400);
  const key = await page.evaluate(() => getComputedStyle(document.querySelector('mathra-keyboard[for="c"]').shadowRoot.querySelector("button")).backgroundColor);
  assert.equal(key, "rgb(47, 51, 54)");
  if (shots) await page.screenshot({ path: `${shots}/dark.png`, fullPage: true });
  await page.selectOption("#theme", "light");
});

await check("Languages: the tooltips and the speech follow `locale`", async () => {
  await page.evaluate(() => (document.getElementById("a").value = "\\frac{1}{2}"));
  await page.selectOption("#locale", "en");
  await page.waitForFunction(() => document.getElementById("a").toolbar.shadowRoot.querySelector('[role=tab]')?.textContent === "Structures");
  const speech = await page.evaluate(() => document.getElementById("a").field.output?.speech ?? "");
  assert.match(speech, /fraction|over/i);
  await page.selectOption("#locale", "pt-BR");
});

await check("No error in the console", async () => {
  assert.deepEqual(errors, []);
});

await context.close();

// a phone: the keyboard starts open, nothing wider than the screen
const phone = await browser.newContext({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 3, isMobile: true, hasTouch: true });
const p2 = await phone.newPage();
await p2.goto(url);
await p2.waitForFunction(() => document.getElementById("a")?.field?.shadowRoot?.querySelector("svg"));
await check("Phone: the editor's keyboard starts open, and the page does not scroll sideways", async () => {
  assert.equal(await p2.evaluate(() => document.getElementById("a").keyboard.hidden), false);
  const wide = await p2.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  assert.ok(wide <= 0, `${wide}px too wide`);
  if (shots) await p2.screenshot({ path: `${shots}/phone.png` });
});
await check("Phone: tapping keys builds a formula", async () => {
  const kb = p2.locator("#a").locator("mathra-keyboard");
  await p2.evaluate(() => (document.getElementById("a").value = ""));
  await kb.locator('button[title="Raiz quadrada"]').tap();
  await kb.locator('button[title="2"]').tap();
  assert.equal(await p2.evaluate(() => document.getElementById("a").value), "\\sqrt{2}");
});
await phone.close();

await browser.close();
server.close();
console.log(results.join("\n"));
process.exit(results.some((r) => r.startsWith("FAIL")) ? 1 : 0);
