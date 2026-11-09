// What the components share: the theme's variables, the icons, the words in each language, and how a
// component finds the field it works for.

import type { MathraField } from "./field.ts";

/** The theme: every color and size the components use. Set `--mathra-<name>` on an element or any
 * ancestor to change it (it reaches the components inside `<mathra-editor>` too); the defaults use
 * `light-dark()`, which follows the page's `color-scheme`. */
export const TOKENS = `
:host {
  --_ink: var(--mathra-ink, light-dark(#1e2224, #ece9e1));
  --_muted: var(--mathra-muted, light-dark(#5c6062, #a8abaa));
  --_background: var(--mathra-background, light-dark(#ffffff, #202326));
  --_surface: var(--mathra-surface, light-dark(#f4f3ef, #2a2e31));
  --_border: var(--mathra-border, light-dark(#d4d1c7, #42474b));
  --_focus: var(--mathra-focus, light-dark(#1f5f99, #9fd3f2));
  --_caret: var(--mathra-caret, light-dark(#1f5f99, #9fd3f2));
  --_selection: var(--mathra-selection, light-dark(rgb(31 95 153 / 0.18), rgb(159 211 242 / 0.25)));
  --_slot: var(--mathra-slot, light-dark(#a5713f, #d9a873));
  --_key: var(--mathra-key, light-dark(#ffffff, #2f3336));
  --_key-active: var(--mathra-key-active, light-dark(#ecebe6, #3a3f43));
  --_accent: var(--mathra-accent, light-dark(#1f5f99, #9fd3f2));
  --_radius: var(--mathra-radius, 6px);
  --_font: var(--mathra-font, system-ui, -apple-system, "Segoe UI", sans-serif);
  --_mono: var(--mathra-mono, ui-monospace, "SF Mono", Menlo, Consolas, monospace);
  font-family: var(--_font);
}
`;

/** Icons (24×24, stroked, in the style of Lucide). */
const ICON_PATHS: Record<string, string> = {
  left: "M15 18l-6-6 6-6",
  right: "M9 18l6-6-6-6",
  up: "M18 15l-6-6-6 6",
  down: "M6 9l6 6 6-6",
  backspace: "M21 4H8l-7 8 7 8h13a2 2 0 0 0 2-2V6a2 2 0 0 0-2-2z M18 9l-6 6 M12 9l6 6",
  enter: "M9 10l-5 5 5 5 M20 4v7a4 4 0 0 1-4 4H4",
  copy: "M8 8h12v12H8z M16 8V4H4v12h4",
  check: "M20 6L9 17l-5-5",
  download: "M12 3v12 M7 10l5 5 5-5 M5 21h14",
  undo: "M9 14L4 9l5-5 M4 9h10.5a5.5 5.5 0 0 1 0 11H11",
  redo: "M15 14l5-5-5-5 M20 9H9.5a5.5 5.5 0 0 0 0 11H13",
  shift: "M12 3l9 9h-5v8H8v-8H3z",
  tab: "M21 4v16 M3 12h14 M13 8l4 4-4 4",
  rowAdd: "M3 5h18 M3 11h18 M12 15v6 M9 18h6",
  rowRemove: "M3 5h18 M3 11h18 M9 18h6",
  columnAdd: "M5 3v18 M11 3v18 M18 9v6 M15 12h6",
  columnRemove: "M5 3v18 M11 3v18 M15 12h6",
  close: "M18 6L6 18 M6 6l12 12",
  keyboard: "M3 6h18v12H3z M7 10h0 M11 10h0 M15 10h0 M8 14h8",
  clear: "M3 6h18 M8 6V4h8v2 M6 6l1 14h10l1-14",
};

/** An icon as inline SVG; the button around it carries the name (`title` and `aria-label`). */
export function icon(name: string): string {
  const d = ICON_PATHS[name] ?? "";
  return `<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${d
    .split(" M")
    .map((p, i) => `<path d="${i ? "M" : ""}${p}"/>`)
    .join("")}</svg>`;
}

/** A word in the field's language: an object by locale (falls back to its language, then English) or one string. */
export type Words = string | Record<string, string>;

export function say(words: Words | undefined, locale: string | null | undefined): string {
  if (words === undefined) return "";
  if (typeof words === "string") return words;
  const l = locale ?? "en";
  return words[l] ?? words[l.split("-")[0]] ?? words.en ?? Object.values(words)[0] ?? "";
}

/** The words of the components' own interface. */
export const UI = {
  formula: { en: "Formula", "pt-BR": "Fórmula" },
  copy: { en: "Copy", "pt-BR": "Copiar" },
  copied: { en: "Copied", "pt-BR": "Copiado" },
  download: { en: "Download", "pt-BR": "Baixar" },
  insert: { en: "Insert", "pt-BR": "Inserir" },
  commands: { en: "Commands", "pt-BR": "Comandos" },
  noCommand: { en: "No command with this name", "pt-BR": "Nenhum comando com esse nome" },
  keyboard: { en: "Keyboard", "pt-BR": "Teclado" },
  undo: { en: "Undo", "pt-BR": "Desfazer" },
  redo: { en: "Redo", "pt-BR": "Refazer" },
  clear: { en: "Clear", "pt-BR": "Limpar" },
  left: { en: "Move left", "pt-BR": "Mover para a esquerda" },
  right: { en: "Move right", "pt-BR": "Mover para a direita" },
  up: { en: "Move up", "pt-BR": "Mover para cima" },
  down: { en: "Move down", "pt-BR": "Mover para baixo" },
  backspace: { en: "Delete back", "pt-BR": "Apagar" },
  enter: { en: "Enter", "pt-BR": "Confirmar" },
  tab: { en: "Next empty place", "pt-BR": "Próximo lugar vazio" },
  shift: { en: "Capitals", "pt-BR": "Maiúsculas" },
  latex: { en: "LaTeX", "pt-BR": "LaTeX" },
  mathml: { en: "MathML", "pt-BR": "MathML" },
  svg: { en: "SVG", "pt-BR": "SVG" },
  png: { en: "PNG", "pt-BR": "PNG" },
  speech: { en: "Speech", "pt-BR": "Fala" },
  braille: { en: "Braille", "pt-BR": "Braille" },
  text: { en: "Text", "pt-BR": "Texto" },
  empty: { en: "Nothing to show yet", "pt-BR": "Nada para mostrar ainda" },
  outputs: { en: "Outputs", "pt-BR": "Saídas" },
  palettes: { en: "Palettes", "pt-BR": "Paletas" },
} satisfies Record<string, Words>;

/** Escapes text for HTML. */
export function html(s: string): string {
  return s.replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]!);
}

/** A component that works for a field: by `for="id"` (in the same document or shadow root), by the
 * `field` property, or inside a `<mathra-editor>`. Re-attaches when the field appears later. */
export class FieldClient extends HTMLElement {
  private _field: MathraField | null = null;
  private detach: (() => void) | null = null;
  private observer: MutationObserver | null = null;

  /** The field this component works for. */
  get field(): MathraField | null {
    return this._field;
  }

  set field(f: MathraField | null) {
    if (f === this._field) return;
    this.detach?.();
    this.detach = null;
    this._field = f;
    if (f) this.detach = this.attach(f);
    this.fieldChanged();
  }

  /** The field's locale, or this element's. */
  get locale(): string {
    return this.getAttribute("locale") ?? this._field?.getAttribute("locale") ?? "en";
  }

  connectedCallback(): void {
    this.findField();
    if (!this._field && this.getAttribute("for")) {
      // the field may come later in the page
      this.observer = new MutationObserver(() => {
        this.findField();
        if (this._field) this.observer?.disconnect();
      });
      this.observer.observe(this.getRootNode() as Node, { childList: true, subtree: true });
    }
  }

  disconnectedCallback(): void {
    this.observer?.disconnect();
  }

  private findField(): void {
    if (this._field) return;
    const id = this.getAttribute("for");
    if (!id) return;
    const root = this.getRootNode() as Document | ShadowRoot;
    const el = root.getElementById?.(id) ?? null;
    if (el && el.localName === "mathra-field") this.field = el as MathraField;
  }

  /** Listens to the field; returns what undoes it. */
  protected attach(_field: MathraField): () => void {
    return () => {};
  }

  /** Called when the field is set or changes. */
  protected fieldChanged(): void {}
}

/** Keeps focus in the field when a button is pressed, so typing goes on where it was. */
export function keepFocus(el: HTMLElement): void {
  el.addEventListener("pointerdown", (e) => {
    if ((e.target as HTMLElement).closest("button")) e.preventDefault();
  });
}
