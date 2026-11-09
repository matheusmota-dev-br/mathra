// <mathra-editor>: everything together, for a page that only wants a formula editor: palettes in
// tabs, the field, undo and redo, a keyboard on the screen, the command menu and the outputs.

import { TOKENS, html, icon, keepFocus, say, UI } from "./shared.ts";
import type { MathraField } from "./field.ts";
import type { MathraToolbar } from "./palette.ts";
import type { MathraKeyboard } from "./keyboard.ts";
import type { MathraOutput } from "./output.ts";
import type { MathraCommandMenu } from "./command-menu.ts";
import "./field.ts";
import "./palette.ts";
import "./keyboard.ts";
import "./output.ts";
import "./command-menu.ts";

const STYLE = `
${TOKENS}
:host { display: block; color: var(--_ink); }
:host([hidden]) { display: none; }
.wrap { display: grid; gap: var(--mathra-gap, 10px); }
.actions { display: flex; gap: 4px; justify-content: flex-end; }
.actions button { display: inline-grid; place-items: center; width: 34px; height: 34px; color: var(--_muted); background: none; border: 1px solid transparent; border-radius: var(--_radius); cursor: pointer; transition: color 0.12s, border-color 0.12s; }
.actions button:hover:not(:disabled) { color: var(--_ink); border-color: var(--_border); }
.actions button:disabled { opacity: 0.35; cursor: default; }
.actions button[aria-pressed="true"] { color: var(--_accent); border-color: var(--_accent); }
.actions button:focus-visible { outline: 2px solid var(--_focus); outline-offset: 1px; }
mathra-keyboard { animation: rise 0.2s ease-out; }
@keyframes rise { from { opacity: 0; transform: translateY(6px); } }
@media (prefers-reduced-motion: reduce) { mathra-keyboard { animation: none; } .actions button { transition: none; } }
`;

export class MathraEditor extends HTMLElement {
  static observedAttributes = ["value", "locale", "palettes", "outputs", "keyboard", "readonly", "placeholder", "aria-label"];

  readonly field: MathraField;
  readonly toolbar: MathraToolbar;
  readonly keyboard: MathraKeyboard;
  readonly outputPanel: MathraOutput;
  readonly commandMenu: MathraCommandMenu;
  private actions: HTMLDivElement;

  constructor() {
    super();
    const root = this.attachShadow({ mode: "open" });
    root.innerHTML = `<style>${STYLE}</style><div class="wrap" part="wrap"><slot name="start"></slot><mathra-toolbar part="toolbar" exportparts="tab, button, grid"></mathra-toolbar><mathra-field part="field" exportparts="box, caret, selection, slot, placeholder"></mathra-field><div class="actions" part="actions"></div><mathra-keyboard part="keyboard" exportparts="key" hidden></mathra-keyboard><mathra-output part="output" exportparts="tab: output-tab, action: output-action"></mathra-output><slot name="end"></slot></div><mathra-command-menu></mathra-command-menu>`;
    this.field = root.querySelector("mathra-field")!;
    this.toolbar = root.querySelector("mathra-toolbar")!;
    this.keyboard = root.querySelector("mathra-keyboard")!;
    this.outputPanel = root.querySelector("mathra-output")!;
    this.commandMenu = root.querySelector("mathra-command-menu")!;
    this.actions = root.querySelector(".actions")!;
    for (const c of [this.toolbar, this.keyboard, this.outputPanel, this.commandMenu]) c.field = this.field;
    keepFocus(this.actions);
    this.actions.addEventListener("click", (e) => {
      const b = (e.target as HTMLElement).closest<HTMLButtonElement>("button");
      if (!b) return;
      if (b.dataset.act === "keyboard") {
        this.keyboard.hidden = !this.keyboard.hidden;
        this.paintActions();
      } else this.field.run(b.dataset.act!);
      this.field.focus();
    });
    this.field.addEventListener("input", () => this.paintActions());
  }

  connectedCallback(): void {
    this.sync();
    // on a touch screen the keyboard on the screen starts open, unless keyboard="off"
    const mode = this.getAttribute("keyboard");
    this.keyboard.hidden = mode === "on" ? false : mode === "off" ? true : !matchMedia("(pointer: coarse)").matches;
    this.paintActions();
  }

  attributeChangedCallback(): void {
    if (this.isConnected) this.sync();
  }

  /** The formula as plain LaTeX. */
  get value(): string {
    return this.field.value;
  }

  set value(latex: string) {
    this.field.value = latex;
  }

  /** The editor without a screen. */
  get editor() {
    return this.field.editor;
  }

  override focus(options?: FocusOptions): void {
    this.field.focus(options);
  }

  private sync(): void {
    const pass = (el: HTMLElement, name: string, as = name) => {
      const v = this.getAttribute(name);
      if (v === null) el.removeAttribute(as);
      else el.setAttribute(as, v);
    };
    for (const name of ["value", "locale", "readonly", "placeholder", "aria-label"]) pass(this.field, name);
    for (const el of [this.toolbar, this.keyboard, this.outputPanel, this.commandMenu]) pass(el, "locale");
    pass(this.toolbar, "palettes");
    const outputs = this.getAttribute("outputs");
    this.outputPanel.hidden = outputs === "none";
    if (outputs && outputs !== "none") this.outputPanel.setAttribute("formats", outputs);
    this.toolbar.hidden = this.hasAttribute("readonly") || this.getAttribute("palettes") === "none";
    this.actions.hidden = this.hasAttribute("readonly");
    this.paintActions();
  }

  private paintActions(): void {
    const locale = this.getAttribute("locale") ?? "en";
    const e = this.field.editor;
    const button = (act: string, name: string, words: (typeof UI)[keyof typeof UI], disabled = false, pressed?: boolean) => {
      const label = html(say(words, locale));
      return `<button type="button" part="action" data-act="${act}" title="${label}" aria-label="${label}"${disabled ? " disabled" : ""}${pressed === undefined ? "" : ` aria-pressed="${pressed}"`}>${icon(name)}</button>`;
    };
    this.actions.innerHTML =
      button("undo", "undo", UI.undo, !e.canUndo()) +
      button("redo", "redo", UI.redo, !e.canRedo()) +
      button("clear", "clear", UI.clear, e.isEmpty()) +
      (this.getAttribute("keyboard") === "off" ? "" : button("keyboard", "keyboard", UI.keyboard, false, !this.keyboard.hidden));
  }
}

if (typeof customElements !== "undefined" && !customElements.get("mathra-editor")) customElements.define("mathra-editor", MathraEditor);
