// <mathra-field>: the box where a formula is built. mathra draws it at every change; the boxes it
// returns place the cursor and the selection and read the clicks.

import { init, render, type Options, type Result } from "@mathlabs/mathra";
import { Editor, type ChangeKind } from "./model.ts";
import { boxesById, caretRect, cursorAt, selectionRect, type Box, type Rect } from "./geometry.ts";
import { TOKENS, UI, say } from "./shared.ts";

const STYLE = `
${TOKENS}
:host {
  display: block;
  font-size: 1.5em;
  color: var(--_ink);
  -webkit-tap-highlight-color: transparent;
}
:host([inline]) { display: inline-block; font-size: inherit; vertical-align: middle; }
:host([hidden]) { display: none; }
.box {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: var(--mathra-align, center);
  min-height: 1.6em;
  padding: var(--mathra-padding, 0.6em 0.8em);
  background: var(--_background);
  border: 1px solid var(--_border);
  border-radius: var(--_radius);
  cursor: text;
  overflow-x: auto;
  overflow-y: hidden;
  touch-action: manipulation;
  transition: border-color 0.15s, box-shadow 0.15s;
}
:host([inline]) .box { min-height: 1.3em; padding: var(--mathra-padding, 0.15em 0.35em); min-width: 3em; }
:host(:focus-within) .box { border-color: var(--_focus); box-shadow: 0 0 0 3px color-mix(in srgb, var(--_focus) 22%, transparent); }
:host([readonly]) .box { cursor: default; }
.drawing { position: relative; line-height: 0; flex: none; }
.drawing svg { display: block; overflow: visible; }
g[data-empty] { fill: var(--_slot); }
.overlay { position: absolute; pointer-events: none; }
.caret { background: var(--_caret); border-radius: 1px; animation: blink 1.05s steps(1) infinite; }
.selection { background: var(--_selection); border-radius: 2px; }
:host(:not(:focus-within)) .caret, :host([readonly]) .caret { display: none; }
@keyframes blink { 50% { opacity: 0; } }
@media (prefers-reduced-motion: reduce) { .caret { animation: none; } .box { transition: none; } }
:host([data-empty]:not(:focus-within)) .drawing { visibility: hidden; }
.placeholder { position: absolute; left: var(--mathra-placeholder-left, 0.8em); color: var(--_muted); font-size: 0.6em; pointer-events: none; font-family: var(--_font); }
textarea { position: absolute; opacity: 0; width: 1px; height: 1px; padding: 0; border: 0; margin: 0; resize: none; overflow: hidden; left: 0; top: 0; pointer-events: none; }
.live { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0 0 0 0); white-space: nowrap; }
`;

let ready: Promise<void> | null = null;
const decimalCommaByLocale = new Map<string, boolean>();

/** Does mathra read `1,2` as one number in this locale? Asked once per locale. */
function decimalComma(locale: string): boolean {
  if (!decimalCommaByLocale.has(locale)) {
    const strip = (r: Result) => (r.svg.match(/<use\b[^>]*>/g) ?? []).map((t) => t.replace(/#mathra-[0-9a-f]+-/, "#")).join();
    const o = { locale: locale as Options["locale"] };
    decimalCommaByLocale.set(locale, strip(render("1,2", o)) === strip(render("1{,}2", o)));
  }
  return decimalCommaByLocale.get(locale)!;
}

export interface InputDetail {
  /** The formula as plain LaTeX. */
  latex: string;
}

/** Sent while a command is typed after `\` (`active`), and when it ends: what a command menu follows. */
export interface CommandDetail {
  /** The letters typed so far. */
  name: string;
  active: boolean;
  /** Where the command is on the screen (viewport pixels). */
  rect: DOMRect | null;
}

/** The time mathra and the cursor took for the last change, in milliseconds. */
export interface Timing {
  render: number;
  total: number;
}

/** A key combination (`Mod+B`, `Shift+Enter`, `Alt+ArrowUp`; `Mod` is Ctrl, or ⌘ on a Mac) and the action it runs (see `run`). */
export type Keymap = Record<string, string>;

const IS_MAC = typeof navigator !== "undefined" && /Mac|iPhone|iPad/.test(navigator.platform ?? "");

/** What the keys do, unless `keymap` says otherwise. */
export const DEFAULT_KEYMAP: Keymap = {
  ArrowLeft: "moveLeft",
  ArrowRight: "moveRight",
  ArrowUp: "moveUp",
  ArrowDown: "moveDown",
  "Shift+ArrowLeft": "selectLeft",
  "Shift+ArrowRight": "selectRight",
  Home: "home",
  End: "end",
  "Shift+Home": "selectHome",
  "Shift+End": "selectEnd",
  Backspace: "backspace",
  Delete: "deleteForward",
  Enter: "enter",
  Tab: "tab",
  "Shift+Tab": "tabBack",
  "Mod+A": "selectAll",
  "Mod+Z": "undo",
  "Mod+Shift+Z": "redo",
  "Mod+Y": "redo",
  "Mod+Enter": "addRow",
  "Mod+Shift+Enter": "addColumn",
};

/** The key combination of a keyboard event, in the keymap's notation. */
function comboOf(e: KeyboardEvent): string {
  const parts: string[] = [];
  if (IS_MAC ? e.metaKey : e.ctrlKey) parts.push("Mod");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  parts.push(e.key.length === 1 ? e.key.toUpperCase() : e.key);
  return parts.join("+");
}

export class MathraField extends HTMLElement {
  static observedAttributes = ["locale", "value", "inline", "readonly", "placeholder", "aria-label"];

  /** The editor without a screen: the formula, the cursor, the commands. */
  readonly editor = new Editor();
  /** How long the last change took. */
  lastTiming: Timing = { render: 0, total: 0 };
  /** Keys added to (or replacing) `DEFAULT_KEYMAP`; an empty string turns a key off. */
  keymap: Keymap = {};

  private _mathraOptions: Partial<Options> = {};
  private drawing: HTMLDivElement;
  private caret: HTMLDivElement;
  private selectionBox: HTMLDivElement;
  private input: HTMLTextAreaElement;
  private placeholderEl: HTMLDivElement;
  private live: HTMLDivElement;
  private box: HTMLDivElement;
  private boxes = new Map<string, Box>();
  private plain: Result | null = null;
  private latexAtFocus = "";
  private dragging = false;
  private liveTimer = 0;
  private commandShown = false;
  private ownValue: string | null = null;

  constructor() {
    super();
    const root = this.attachShadow({ mode: "open", delegatesFocus: true });
    root.innerHTML = `<style>${STYLE}</style><div class="box" part="box"><div class="drawing" part="drawing"></div><div class="overlay selection" part="selection" hidden></div><div class="overlay caret" part="caret"></div><div class="placeholder" part="placeholder" hidden></div><textarea part="input" autocapitalize="off" autocomplete="off" autocorrect="off" spellcheck="false"></textarea><div class="live" aria-live="polite"></div></div>`;
    this.box = root.querySelector(".box")!;
    this.drawing = root.querySelector(".drawing")!;
    this.caret = root.querySelector(".caret")!;
    this.selectionBox = root.querySelector(".selection")!;
    this.placeholderEl = root.querySelector(".placeholder")!;
    this.input = root.querySelector("textarea")!;
    this.live = root.querySelector(".live")!;
    this.input.addEventListener("keydown", (e) => this.keydown(e));
    this.input.addEventListener("input", () => this.typed());
    this.input.addEventListener("copy", (e) => this.clipboard(e, "copy"));
    this.input.addEventListener("cut", (e) => this.clipboard(e, "cut"));
    this.input.addEventListener("paste", (e) => this.pasted(e));
    this.input.addEventListener("focus", () => (this.latexAtFocus = this.latex));
    this.input.addEventListener("blur", () => {
      if (this.latex !== this.latexAtFocus) this.dispatchEvent(new CustomEvent<InputDetail>("change", { detail: { latex: this.latex }, bubbles: true, composed: true }));
      this.sendCommand(false);
    });
    this.box.addEventListener("pointerdown", (e) => this.pointerdown(e));
    this.box.addEventListener("pointermove", (e) => this.pointermove(e));
    this.box.addEventListener("pointerup", () => (this.dragging = false));
    this.box.addEventListener("dblclick", () => this.selectRow());
    this.editor.onChange((kind) => this.changed(kind));
    if (typeof ResizeObserver !== "undefined") new ResizeObserver(() => this.placeOverlays()).observe(this.box);
  }

  connectedCallback(): void {
    if (!this.hasAttribute("tabindex")) this.tabIndex = 0;
    ready ??= init();
    ready.then(() => {
      const value = this.getAttribute("value");
      if (value !== null && this.editor.isEmpty()) this.editor.setLatex(value);
      else this.draw();
    });
  }

  attributeChangedCallback(name: string, old: string | null, value: string | null): void {
    if (old === value) return;
    if (name === "value") {
      // a value written back by the page from the field's own `input` changes nothing
      if (value !== null && value !== this.latex && value !== this.ownValue && ready) ready.then(() => this.editor.setLatex(value));
      return;
    }
    if (name === "readonly") this.input.readOnly = value !== null;
    if (this.isConnected && ready) ready.then(() => this.draw());
  }

  // ---------- public API ----------

  /** The formula as plain LaTeX. Setting it replaces the formula (it can be undone). */
  get value(): string {
    return this.editor.toLatex();
  }

  set value(latex: string) {
    if (latex !== this.value) this.editor.setLatex(latex);
  }

  /** The formula as plain LaTeX (the same as `value`). */
  get latex(): string {
    return this.editor.toLatex();
  }

  get locale(): string {
    return this.getAttribute("locale") ?? "en";
  }

  get readOnly(): boolean {
    return this.hasAttribute("readonly");
  }

  /** More options for mathra (a font from `loadFont`, fallback fonts…); the field sets `locale` and `display`. */
  get mathraOptions(): Partial<Options> {
    return this._mathraOptions;
  }

  set mathraOptions(o: Partial<Options>) {
    this._mathraOptions = o;
    if (ready) ready.then(() => this.draw());
  }

  /** Everything mathra gives for the formula: SVG, MathML, speech, braille… (null while it is empty). */
  get output(): Result | null {
    return this.plain;
  }

  /** The options the field draws with. */
  get options(): Options {
    return { ...this._mathraOptions, locale: this.locale as Options["locale"], display: !this.hasAttribute("inline") };
  }

  /** Runs an action, as a key, a palette or a virtual keyboard does:
   * `moveLeft` `moveRight` `moveUp` `moveDown` `selectLeft` `selectRight` `home` `end` `selectHome`
   * `selectEnd` `selectAll` `backspace` `deleteForward` `enter` `tab` `tabBack` `undo` `redo` `clear`
   * `addRow` `addRowAbove` `addColumn` `addColumnLeft` `removeRow` `removeColumn` `sup` `sub`,
   * or `insert:<template>`, `latex:<snippet>`, `type:<text>`, `symbol:<latex>`. */
  run(action: string): void {
    if (this.readOnly) return;
    const e = this.editor;
    const colon = action.indexOf(":");
    const verb = colon < 0 ? action : action.slice(0, colon);
    const arg = colon < 0 ? "" : action.slice(colon + 1);
    const actions: Record<string, () => void> = {
      moveLeft: () => e.moveLeft(),
      moveRight: () => e.moveRight(),
      moveUp: () => e.moveUp(),
      moveDown: () => e.moveDown(),
      selectLeft: () => e.moveLeft(true),
      selectRight: () => e.moveRight(true),
      home: () => e.home(),
      end: () => e.end(),
      selectHome: () => e.home(true),
      selectEnd: () => e.end(true),
      selectAll: () => e.selectAll(),
      backspace: () => e.backspace(),
      deleteForward: () => e.deleteForward(),
      enter: () => e.enter(),
      tab: () => e.tab(),
      tabBack: () => e.tab(true),
      undo: () => e.undo(),
      redo: () => e.redo(),
      clear: () => e.clear(),
      addRow: () => e.addRow(),
      addRowAbove: () => e.addRow(true),
      addColumn: () => e.addColumn(),
      addColumnLeft: () => e.addColumn(true),
      removeRow: () => e.removeRow(),
      removeColumn: () => e.removeColumn(),
      sup: () => e.insert("sup"),
      sub: () => e.insert("sub"),
      insert: () => e.insert(arg),
      latex: () => e.insertLatex(arg, { autoDelimiters: true }),
      type: () => e.typeText(arg),
      symbol: () => e.insertSymbol(arg),
    };
    const f = actions[verb];
    if (!f) throw new Error(`mathra-field: no action named ${verb}`);
    f();
  }

  /** Inserts a structure by name (`frac`, `sqrt`, `pmatrix`, `sup`…). */
  insert(template: string): void {
    this.run(template === "sup" || template === "sub" ? template : `insert:${template}`);
  }

  /** Inserts a snippet of LaTeX (`#?` an empty place, `#0` the selection). */
  insertLatex(snippet: string): void {
    this.run(`latex:${snippet}`);
  }

  /** Inserts a symbol: `\alpha`, `\pm`… */
  insertSymbol(latex: string, cls?: Parameters<Editor["insertSymbol"]>[1]): void {
    if (!this.readOnly) this.editor.insertSymbol(latex, cls);
  }

  override focus(options?: FocusOptions): void {
    this.input.focus(options);
  }

  override blur(): void {
    this.input.blur();
  }

  // ---------- drawing ----------

  private changed(kind: ChangeKind): void {
    if (kind === "content") {
      this.draw();
      this.ownValue = this.latex;
      this.dispatchEvent(new CustomEvent<InputDetail>("input", { detail: { latex: this.latex }, bubbles: true, composed: true }));
    } else this.placeOverlays();
    this.sendCommand(!!this.editor.commandBeingTyped());
  }

  private draw(): void {
    const t0 = performance.now();
    const locale = this.locale;
    this.editor.decimalComma = decimalComma(locale);
    const r = render(this.editor.toLatex({ marked: true }), this.options);
    const t1 = performance.now();
    this.drawing.innerHTML = r.svg;
    // the field's description carries the speech: the drawing itself is hidden from screen readers
    this.drawing.querySelector("svg")?.setAttribute("aria-hidden", "true");
    this.boxes = boxesById(r.boxes as Box[]);
    for (const g of this.drawing.querySelectorAll("g[data-empty]")) g.setAttribute("part", "slot");
    // the plain formula: its speech (marks would make mathra read b² as "b to the power of 2") and the outputs
    const plain = this.editor.toLatex();
    this.plain = plain ? render(plain, this.options) : null;
    this.input.setAttribute("aria-label", this.getAttribute("aria-label") ?? say(UI.formula, locale));
    this.input.setAttribute("aria-description", this.plain?.speech ?? "");
    this.input.setAttribute("aria-roledescription", "math");
    this.placeholderEl.hidden = !this.editor.isEmpty() || !this.getAttribute("placeholder");
    // with a placeholder, the empty place shows only once the field has the focus
    this.toggleAttribute("data-empty", this.editor.isEmpty() && !!this.getAttribute("placeholder"));
    this.placeholderEl.textContent = this.getAttribute("placeholder") ?? "";
    this.placeOverlays();
    this.announce();
    this.lastTiming = { render: t1 - t0, total: performance.now() - t0 };
  }

  /** From the SVG's coordinates to pixels inside the box. */
  private toBox(rect: Rect): { left: number; top: number; width: number; height: number } | null {
    const svg = this.drawing.querySelector("svg");
    if (!svg) return null;
    const vb = svg.viewBox.baseVal;
    const s = svg.getBoundingClientRect();
    if (!vb.width || !s.width) return null;
    const k = s.width / vb.width;
    const b = this.box.getBoundingClientRect();
    return { left: s.left - b.left + this.box.scrollLeft + (rect.x - vb.x) * k, top: s.top - b.top + (rect.y - vb.y) * k, width: rect.width * k, height: rect.height * k };
  }

  private placeOverlays(): void {
    const caret = caretRect(this.editor.cursor, this.boxes);
    const p = caret && this.toBox(caret);
    if (p) {
      Object.assign(this.caret.style, { left: `${p.left}px`, top: `${p.top}px`, width: `${Math.max(1.5, Math.min(p.width, 2.5))}px`, height: `${p.height}px` });
      // restart the blink, so the cursor shows right after it moves
      this.caret.style.animation = "none";
      void this.caret.offsetWidth;
      this.caret.style.animation = "";
      // keep the cursor in view in a long formula
      const left = p.left - this.box.scrollLeft;
      if (left < 0 || left > this.box.clientWidth) this.box.scrollLeft += left - this.box.clientWidth / 2;
      // the hidden input sits at the cursor, so a phone's keyboard and IME windows open there
      Object.assign(this.input.style, { left: `${p.left}px`, top: `${p.top}px` });
    }
    const sel = selectionRect(this.editor.selection(), this.boxes);
    const q = sel && this.toBox(sel);
    this.selectionBox.hidden = !q;
    if (q) Object.assign(this.selectionBox.style, { left: `${q.left}px`, top: `${q.top}px`, width: `${q.width}px`, height: `${q.height}px` });
  }

  /** Tells screen readers the formula, a moment after typing stops. */
  private announce(): void {
    clearTimeout(this.liveTimer);
    this.liveTimer = window.setTimeout(() => {
      if (this.matches(":focus-within")) this.live.textContent = this.plain?.speech ?? "";
    }, 700);
  }

  /** Tells a command menu what is being typed after `\`, and where. */
  private sendCommand(active: boolean): void {
    if (!active && !this.commandShown) return;
    this.commandShown = active;
    const command = this.editor.commandBeingTyped();
    let rect: DOMRect | null = null;
    if (active && command) {
      const b = this.boxes.get(command.id);
      const p = b && this.toBox(b);
      const box = this.box.getBoundingClientRect();
      if (p) rect = new DOMRect(box.left + p.left - this.box.scrollLeft, box.top + p.top, p.width, p.height);
    }
    this.dispatchEvent(new CustomEvent<CommandDetail>("mathra-command", { detail: { name: command?.name ?? "", active, rect }, bubbles: true, composed: true }));
  }

  // ---------- input ----------

  private pointerdown(e: PointerEvent): void {
    if (e.button !== 0) return;
    e.preventDefault();
    this.input.focus();
    const at = this.pointAt(e);
    if (!at || this.readOnly) return;
    this.editor.setCursor(at, e.shiftKey);
    this.dragging = true;
    this.box.setPointerCapture?.(e.pointerId);
  }

  private pointermove(e: PointerEvent): void {
    if (!this.dragging || !(e.buttons & 1)) return;
    const at = this.pointAt(e);
    if (at) this.editor.setCursor(at, true);
  }

  private pointAt(e: PointerEvent) {
    const svg = this.drawing.querySelector("svg");
    if (!svg) return null;
    const vb = svg.viewBox.baseVal;
    const b = svg.getBoundingClientRect();
    if (!b.width) return null;
    const k = b.width / vb.width;
    return cursorAt(this.editor, this.boxes, vb.x + (e.clientX - b.left) / k, vb.y + (e.clientY - b.top) / k);
  }

  /** Double click: all of the row the cursor is in. */
  private selectRow(): void {
    const { row } = this.editor.cursor;
    if (row.children.length && !this.readOnly) this.editor.select(row, 0, row.children.length);
  }

  private keydown(e: KeyboardEvent): void {
    // a command menu, when open, takes the keys it needs first
    const ask = new CustomEvent("mathra-key", { detail: { key: e.key, shift: e.shiftKey }, cancelable: true, bubbles: true, composed: true });
    if (!this.dispatchEvent(ask)) {
      e.preventDefault();
      return;
    }
    if (e.isComposing) return;
    const action = { ...DEFAULT_KEYMAP, ...this.keymap }[comboOf(e)];
    if (action === "") return;
    if (action) {
      // Tab leaves an empty field, so a form can be crossed with it
      if ((action === "tab" || action === "tabBack") && this.editor.isEmpty()) return;
      e.preventDefault();
      this.run(action);
      return;
    }
    if (e.key === "Escape") {
      // leave a command being typed, else the selection
      if (this.editor.commandBeingTyped()) this.editor.backspace();
      else if (this.editor.anchor) this.editor.setCursor(this.editor.cursor);
    }
    // printable characters arrive through `input`, which also covers phones and dead keys
  }

  private typed(): void {
    const text = this.input.value;
    this.input.value = "";
    if (!this.readOnly && text) this.editor.typeText(text);
  }

  private clipboard(e: ClipboardEvent, kind: "copy" | "cut"): void {
    const latex = this.editor.selection() ? this.editor.selectedLatex() : this.latex;
    if (!latex) return;
    e.preventDefault();
    e.clipboardData?.setData("text/plain", latex);
    if (kind === "cut" && !this.readOnly && this.editor.selection()) this.editor.cut();
  }

  private pasted(e: ClipboardEvent): void {
    e.preventDefault();
    const text = e.clipboardData?.getData("text/plain") ?? "";
    if (text && !this.readOnly) this.editor.paste(text);
  }
}

if (typeof customElements !== "undefined" && !customElements.get("mathra-field")) customElements.define("mathra-field", MathraField);
