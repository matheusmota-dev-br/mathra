// <mathra-field>: the box where a formula is built. mathra draws it at every change; the boxes it
// returns place the cursor and read the clicks.

import { init, render, type Options } from "@mathlabs/mathra";
import { Editor, type Template } from "./model.ts";
import { boxesById, caretRect, cursorAt, type Box } from "./geometry.ts";

const STYLE = `
:host {
  --mathra-ink: currentColor;
  --mathra-background: transparent;
  --mathra-border: #c9c5ba;
  --mathra-focus: #1f5f99;
  --mathra-caret: #1f5f99;
  --mathra-slot: #a5713f;
  --mathra-radius: 6px;
  --mathra-padding: 0.6em 0.8em;
  display: inline-block;
  font-size: 1.5em;
  min-width: 4em;
  vertical-align: middle;
}
.box {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 1.6em;
  padding: var(--mathra-padding);
  color: var(--mathra-ink);
  background: var(--mathra-background);
  border: 1px solid var(--mathra-border);
  border-radius: var(--mathra-radius);
  cursor: text;
  overflow-x: auto;
}
:host(:focus-within) .box { border-color: var(--mathra-focus); box-shadow: 0 0 0 2px color-mix(in srgb, var(--mathra-focus) 25%, transparent); }
.drawing { position: relative; line-height: 0; }
.drawing svg { display: block; }
g[data-empty] { fill: var(--mathra-slot); }
.caret { position: absolute; background: var(--mathra-caret); pointer-events: none; animation: blink 1.05s steps(1) infinite; }
:host(:not(:focus-within)) .caret { display: none; }
@keyframes blink { 50% { opacity: 0; } }
@media (prefers-reduced-motion: reduce) { .caret { animation: none; } }
textarea { position: absolute; opacity: 0; width: 1px; height: 1px; padding: 0; border: 0; resize: none; overflow: hidden; left: 0; top: 0; }
`;

let ready: Promise<void> | null = null;

export interface ChangeDetail {
  /** The formula as plain LaTeX. */
  latex: string;
  /** What a screen reader says, in the field's language. */
  speech: string;
}

/** The time mathra and the cursor took for the last change, in milliseconds. */
export interface Timing {
  render: number;
  total: number;
}

export class MathraField extends HTMLElement {
  static observedAttributes = ["locale", "aria-label"];

  /** The editor without a screen: the formula, the cursor, the commands. */
  readonly editor = new Editor();
  /** How long the last change took. */
  lastTiming: Timing = { render: 0, total: 0 };

  private drawing!: HTMLDivElement;
  private caret!: HTMLDivElement;
  private input!: HTMLTextAreaElement;
  private boxes = new Map<string, Box>();
  private speech = "";

  constructor() {
    super();
    const root = this.attachShadow({ mode: "open", delegatesFocus: true });
    root.innerHTML = `<style>${STYLE}</style><div class="box" part="box"><div class="drawing" part="drawing"></div><div class="caret" part="caret"></div><textarea part="input" autocapitalize="off" autocomplete="off" spellcheck="false"></textarea></div>`;
    this.drawing = root.querySelector(".drawing")!;
    this.caret = root.querySelector(".caret")!;
    this.input = root.querySelector("textarea")!;
    this.input.addEventListener("keydown", (e) => this.keydown(e));
    this.input.addEventListener("input", () => this.typed());
    root.querySelector(".box")!.addEventListener("pointerdown", (e) => this.pointerdown(e as PointerEvent));
    this.editor.onChange(() => this.draw());
  }

  connectedCallback(): void {
    ready ??= init();
    ready.then(() => this.draw());
    if (!this.hasAttribute("tabindex")) this.tabIndex = 0;
  }

  attributeChangedCallback(): void {
    if (this.isConnected) ready?.then(() => this.draw());
  }

  /** The formula as plain LaTeX. */
  get latex(): string {
    return this.editor.toLatex();
  }

  /** Inserts a structure at the cursor (what a palette button does). */
  insert(template: Template): void {
    this.editor.insert(template);
    this.input.focus();
  }

  /** Inserts a symbol at the cursor: `\alpha`, `\pm` (class `bin`)… */
  insertSymbol(latex: string, cls?: Parameters<Editor["insertSymbol"]>[1]): void {
    this.editor.insertSymbol(latex, cls);
    this.input.focus();
  }

  /** Options passed to mathra; the field sets `display`. */
  get options(): Options {
    return { locale: (this.getAttribute("locale") as Options["locale"]) ?? undefined };
  }

  private draw(): void {
    const t0 = performance.now();
    const marked = this.editor.toLatex({ marked: true });
    const r = render(marked, { ...this.options, display: true });
    const t1 = performance.now();
    this.drawing.innerHTML = r.svg;
    this.boxes = boxesById(r.boxes as Box[]);
    // the speech of the plain LaTeX: the marks would make mathra read b² as "b to the power of 2"
    const plain = this.editor.toLatex();
    this.speech = plain ? render(plain, { ...this.options, display: true }).speech : "";
    for (const g of this.drawing.querySelectorAll("g[data-empty]")) g.setAttribute("part", "slot");
    this.placeCaret();
    this.setAttribute("aria-description", this.speech);
    this.lastTiming = { render: t1 - t0, total: performance.now() - t0 };
    this.dispatchEvent(new CustomEvent<ChangeDetail>("change", { detail: { latex: this.latex, speech: this.speech }, bubbles: true, composed: true }));
  }

  /** From the SVG's coordinates to pixels inside the drawing. */
  private scale(): { k: number; x0: number; y0: number } | null {
    const svg = this.drawing.querySelector("svg");
    if (!svg) return null;
    const vb = svg.viewBox.baseVal;
    const width = svg.getBoundingClientRect().width;
    return { k: width / vb.width, x0: vb.x, y0: vb.y };
  }

  private placeCaret(): void {
    const s = this.scale();
    const rect = caretRect(this.editor.cursor, this.boxes);
    if (!s || !rect) return;
    const svg = this.drawing.querySelector("svg")!;
    const box = this.drawing.parentElement!.getBoundingClientRect();
    const svgBox = svg.getBoundingClientRect();
    const left = svgBox.left - box.left + (rect.x - s.x0) * s.k;
    const top = svgBox.top - box.top + (rect.y - s.y0) * s.k;
    Object.assign(this.caret.style, { left: `${left}px`, top: `${top}px`, width: `${Math.max(1.5, rect.width * s.k)}px`, height: `${rect.height * s.k}px` });
    // restart the blink, so the cursor shows right after it moves
    this.caret.style.animation = "none";
    void this.caret.offsetWidth;
    this.caret.style.animation = "";
  }

  private pointerdown(e: PointerEvent): void {
    const svg = this.drawing.querySelector("svg");
    const s = this.scale();
    e.preventDefault();
    this.input.focus();
    if (!svg || !s) return;
    const b = svg.getBoundingClientRect();
    const x = s.x0 + (e.clientX - b.left) / s.k;
    const y = s.y0 + (e.clientY - b.top) / s.k;
    this.editor.setCursor(cursorAt(this.editor, this.boxes, x, y));
  }

  private keydown(e: KeyboardEvent): void {
    const ed = this.editor;
    const mod = e.metaKey || e.ctrlKey;
    if (mod && e.key.toLowerCase() === "z") {
      e.preventDefault();
      if (e.shiftKey) ed.redo();
      else ed.undo();
      return;
    }
    if (mod && e.key.toLowerCase() === "y") {
      e.preventDefault();
      ed.redo();
      return;
    }
    if (mod) return;
    const actions: Record<string, () => void> = {
      ArrowLeft: () => ed.moveLeft(),
      ArrowRight: () => ed.moveRight(),
      ArrowUp: () => ed.moveUp(),
      ArrowDown: () => ed.moveDown(),
      Home: () => ed.home(),
      End: () => ed.end(),
      Backspace: () => ed.backspace(),
      Enter: () => ed.enter(),
    };
    const action = actions[e.key];
    if (action) {
      e.preventDefault();
      action();
    }
    // printable characters arrive through `input`, which also covers phones and dead keys
  }

  private typed(): void {
    const text = this.input.value;
    this.input.value = "";
    this.editor.typeText(text);
  }
}

if (typeof customElements !== "undefined" && !customElements.get("mathra-field")) customElements.define("mathra-field", MathraField);
