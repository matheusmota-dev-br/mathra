// The formula being edited: a tree of rows and atoms, a cursor and a selection, and the commands
// that change them. No DOM and no mathra here: `toLatex` writes the LaTeX that mathra draws, with
// each atom and each row marked (`\htmlData{m=<id>}{…}`) so the boxes mathra returns tell where
// everything sits.

import { readLatex, type AtomSpec } from "./latex.ts";
import { CHAR_CLASS, CHAR_LATEX, CLOSING, DELIMITERS, LOWER_LIMIT_ONLY, classOfCommand, escapeText, type AtomClass } from "./symbols.ts";

export type { AtomClass } from "./symbols.ts";
export type { AtomSpec } from "./latex.ts";

/** A horizontal list: the whole formula, a numerator, an exponent, a cell… */
export interface Row {
  kind: "row";
  id: string;
  children: Atom[];
  /** The atom this row belongs to; none for the formula itself. */
  parent: Atom | null;
}

interface Base {
  id: string;
  /** The row this atom is in. */
  parent: Row | null;
}

/** One symbol: a letter, a digit, an operator, `\alpha`, `\sin`, a space… */
export interface SymbolAtom extends Base { kind: "symbol"; latex: string; cls: AtomClass }
/** One character of text (inside `\text{…}`). */
export interface CharAtom extends Base { kind: "char"; ch: string }
export interface TextAtom extends Base { kind: "text"; cmd: string; body: Row }
/** `\frac`, `\dfrac`, `\tfrac`, `\cfrac`, `\binom`… */
export interface Fraction extends Base { kind: "frac"; cmd: string; num: Row; den: Row }
export interface Root extends Base { kind: "sqrt"; index: Row | null; body: Row }
/** An exponent, an index, or both, on the atom before it. */
export interface Scripts extends Base { kind: "scripts"; sup: Row | null; sub: Row | null }
/** Delimiters around a row: they grow (`\left…\right`) only when what they hold is tall. */
export interface Delimited extends Base { kind: "delim"; left: string; right: string; body: Row; auto: boolean }
/** A large operator with its limits: `\sum`, `\int`, `\lim`… */
export interface LargeOperator extends Base { kind: "bigop"; op: string; sub: Row | null; sup: Row | null }
/** `\vec`, `\hat`, `\overline`… over a row. */
export interface Accent extends Base { kind: "accent"; cmd: string; body: Row }
/** `\mathbf`, `\mathbb`, `\operatorname`… over a row. */
export interface Styled extends Base { kind: "style"; cmd: string; body: Row }
/** A grid of cells: matrices, `cases`, `aligned`. */
export interface Matrix extends Base { kind: "matrix"; env: string; cells: Row[][] }
/** LaTeX the editor does not take apart: drawn as written, moved and deleted whole. */
export interface Raw extends Base { kind: "raw"; latex: string }
/** A command being typed after `\`, until a space or Enter turns it into what it names. */
export interface Command extends Base { kind: "command"; name: string }

export type Atom = SymbolAtom | CharAtom | TextAtom | Fraction | Root | Scripts | Delimited | LargeOperator | Accent | Styled | Matrix | Raw | Command;

/** Where the cursor is: between two atoms of a row, `offset` atoms from its start. */
export interface Cursor {
  row: Row;
  offset: number;
}

/** A selection: atoms `start` to `end` (not included) of one row. */
export interface Selection {
  row: Row;
  start: number;
  end: number;
}

/** The structures a name inserts (`insert("frac")`, `\frac` typed, a palette's button). */
export const TEMPLATES: Record<string, string> = {
  frac: "\\frac{#0}{#?}",
  dfrac: "\\dfrac{#0}{#?}",
  tfrac: "\\tfrac{#0}{#?}",
  cfrac: "\\cfrac{#0}{#?}",
  binom: "\\binom{#0}{#?}",
  sqrt: "\\sqrt{#0}",
  nroot: "\\sqrt[#?]{#0}",
  root: "\\sqrt[#?]{#0}",
  paren: "\\left(#0\\right)",
  brackets: "\\left[#0\\right]",
  braces: "\\left\\{#0\\right\\}",
  abs: "\\left|#0\\right|",
  norm: "\\left\\|#0\\right\\|",
  angle: "\\left\\langle #0\\right\\rangle",
  floor: "\\left\\lfloor #0\\right\\rfloor",
  ceil: "\\left\\lceil #0\\right\\rceil",
  sum: "\\sum_{#?}^{#?}#0",
  prod: "\\prod_{#?}^{#?}#0",
  coprod: "\\coprod_{#?}^{#?}#0",
  int: "\\int_{#?}^{#?}#0",
  iint: "\\iint_{#?}#0",
  iiint: "\\iiint_{#?}#0",
  oint: "\\oint_{#?}#0",
  bigcup: "\\bigcup_{#?}^{#?}#0",
  bigcap: "\\bigcap_{#?}^{#?}#0",
  lim: "\\lim_{#?}#0",
  limsup: "\\limsup_{#?}#0",
  liminf: "\\liminf_{#?}#0",
  max: "\\max_{#?}#0",
  min: "\\min_{#?}#0",
  sup_: "\\sup_{#?}#0",
  inf: "\\inf_{#?}#0",
  text: "\\text{#0}",
  matrix: "\\begin{matrix}#?&#?\\\\#?&#?\\end{matrix}",
  pmatrix: "\\begin{pmatrix}#?&#?\\\\#?&#?\\end{pmatrix}",
  bmatrix: "\\begin{bmatrix}#?&#?\\\\#?&#?\\end{bmatrix}",
  Bmatrix: "\\begin{Bmatrix}#?&#?\\\\#?&#?\\end{Bmatrix}",
  vmatrix: "\\begin{vmatrix}#?&#?\\\\#?&#?\\end{vmatrix}",
  Vmatrix: "\\begin{Vmatrix}#?&#?\\\\#?&#?\\end{Vmatrix}",
  cases: "\\begin{cases}#?&#?\\\\#?&#?\\end{cases}",
  aligned: "\\begin{aligned}#?&#?\\\\#?&#?\\end{aligned}",
};
for (const a of ["vec", "hat", "bar", "dot", "ddot", "tilde", "widehat", "widetilde", "overline", "underline", "overrightarrow", "overleftarrow", "overbrace", "underbrace", "boxed", "cancel"]) TEMPLATES[a] = `\\${a}{#0}`;
for (const s of ["mathbf", "mathbb", "mathcal", "mathscr", "mathfrak", "mathrm", "mathit", "mathsf", "mathtt", "boldsymbol"]) TEMPLATES[s] = `\\${s}{#0}`;

/** Names of the structures that attach to the atom before the cursor. */
export type ScriptTemplate = "sup" | "sub";

/** Wrappers that keep a marked symbol's class: a group alone is an ordinary atom, as in TeX. */
const CLASS_WRAPPER: Record<AtomClass, string | null> = { ord: null, bin: "\\mathbin", rel: "\\mathrel", open: "\\mathopen", close: "\\mathclose", punct: "\\mathpunct", op: null };

export interface LatexOptions {
  /** Mark every atom and row with `\htmlData{m=<id>}`, and draw empty rows as `\square`: what the editor draws. Off: the plain LaTeX to hand out. */
  marked?: boolean;
}

export type ChangeKind = "content" | "cursor";
export type ChangeListener = (kind: ChangeKind) => void;

interface Snapshot {
  root: AtomSpec[];
  cursor: [number, number][];
  offset: number;
  anchor: [number, number][] | null;
  anchorOffset: number;
}

/** An editor without a screen: the formula, the cursor, the selection and the commands. */
export class Editor {
  root: Row;
  cursor: Cursor;
  /** The other end of the selection; none when nothing is selected. */
  anchor: Cursor | null = null;
  /** The locale writes `3,14` for a decimal (pt-BR, fr, de…): a comma between digits is then part of the number. The field sets it from mathra. */
  decimalComma = false;
  private next = 0;
  private listeners = new Set<ChangeListener>();
  private undoStack: string[] = [];
  private redoStack: string[] = [];
  private batching = 0;

  constructor(latex = "") {
    this.root = this.row(null);
    this.cursor = { row: this.root, offset: 0 };
    if (latex) {
      this.adoptInto(this.root, readLatex(latex), []);
      this.cursor = { row: this.root, offset: this.root.children.length };
    }
  }

  // ---------- reading ----------

  /** The formula as LaTeX: plain to hand out, or marked to draw with mathra. */
  toLatex(options: LatexOptions = {}): string {
    return this.rowLatex(this.root, options.marked ?? false);
  }

  /** Every row, in document order (outer before inner, left to right). */
  rows(): Row[] {
    const out: Row[] = [];
    const walk = (r: Row) => {
      out.push(r);
      for (const a of r.children) for (const c of childRows(a)) walk(c);
    };
    walk(this.root);
    return out;
  }

  isEmpty(): boolean {
    return this.root.children.length === 0;
  }

  /** The selection, as atoms of one row; none when nothing is selected. */
  selection(): Selection | null {
    if (!this.anchor) return null;
    const a = this.chain(this.anchor);
    const c = this.chain(this.cursor);
    let i = 0;
    while (i + 1 < a.length && i + 1 < c.length && a[i + 1].row === c[i + 1].row) i++;
    const row = a[i].row;
    const edges = [a[i].start, a[i].end, c[i].start, c[i].end];
    const start = Math.min(...edges);
    const end = Math.max(...edges);
    return start === end ? null : { row, start, end };
  }

  /** The selection as plain LaTeX (empty when nothing is selected). */
  selectedLatex(): string {
    const s = this.selection();
    if (!s) return "";
    return this.listLatex(s.row.children.slice(s.start, s.end), false);
  }

  /** The command being typed after `\`, if any. */
  commandBeingTyped(): Command | null {
    const { row, offset } = this.cursor;
    const before = row.children[offset - 1];
    return before && before.kind === "command" ? before : null;
  }

  /** Is the cursor in text (inside `\text{…}`)? */
  inText(): boolean {
    const owner = this.cursor.row.parent;
    return !!owner && owner.kind === "text";
  }

  /** The matrix around the cursor, with the cell it is in. */
  matrixAtCursor(): { matrix: Matrix; i: number; j: number } | null {
    for (let row: Row = this.cursor.row; row.parent; row = row.parent.parent!) {
      const owner = row.parent;
      if (owner.kind === "matrix") {
        for (let i = 0; i < owner.cells.length; i++) {
          const j = owner.cells[i].indexOf(row);
          if (j >= 0) return { matrix: owner, i, j };
        }
      }
    }
    return null;
  }

  /** Called after every change: `content` when the formula changed, `cursor` when only the cursor or the selection moved. Returns a function that removes it. */
  onChange(listener: ChangeListener): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  // ---------- whole formula ----------

  /** Replaces the formula with this LaTeX; the cursor goes to the end. */
  setLatex(latex: string): void {
    this.change(() => {
      this.root.children = [];
      this.adoptInto(this.root, readLatex(latex), []);
      this.cursor = { row: this.root, offset: this.root.children.length };
      this.anchor = null;
    });
  }

  /** Empties the formula. */
  clear(): void {
    this.setLatex("");
  }

  // ---------- typing ----------

  /** Types one character, as a keyboard does. In math: letters, digits and operators become symbols;
   * `/` makes a fraction of what is before the cursor; `^` and `_` an exponent and an index; `(`, `[`,
   * `{` and `|` open delimiters that grow, and their closing character leaves them; `\` starts a
   * command that a space or Enter finishes. In text, every character is itself. */
  type(ch: string): void {
    if (this.inText() && !this.commandBeingTyped()) {
      if (ch === "\n") return;
      return this.change(() => this.replaceSelectionWith([{ kind: "char", ch }]));
    }
    const command = this.commandBeingTyped();
    if (command) {
      if (/^[A-Za-z]$/.test(ch)) return this.change(() => void (command.name += ch));
      // `\` then one other character: the command is that character (`\,`, `\{`, `\|`)
      if (command.name === "" && ",:;! {}|#$%&_".includes(ch) && ch !== " ") {
        return this.change(() => {
          const row = command.parent!;
          const at = row.children.indexOf(command);
          row.children.splice(at, 1);
          this.cursor = { row, offset: at };
          this.insertSpecs(readLatex(`\\${ch}`));
        });
      }
      this.change(() => this.finishCommand(command));
      if (ch === " ") return;
    }
    if (ch === " " || ch === "\n") return;
    if (ch === "\\") return this.change(() => this.replaceSelectionWith([{ kind: "command", name: "" }]));
    if (ch === "/") return this.change(() => this.fractionOfOperand());
    if (ch === "^") return this.script("sup");
    if (ch === "_") return this.script("sub");
    if (ch === "&" || ch === "\t") return this.nextCell();
    const opener = ({ "(": "(", "[": "[", "{": "\\{", "|": "|" } as Record<string, string>)[ch];
    const closer = ({ ")": ")", "]": "]", "}": "\\}", "|": "|" } as Record<string, string>)[ch];
    if (closer && this.leaveDelimiter(closer)) return;
    if (opener) return this.change(() => this.insertSpecs(readLatex(`\\left${opener}#0\\right${CLOSING[opener]}`, { autoDelimiters: true })));
    if (/^[A-Za-z0-9.]$/.test(ch)) return this.insertSymbol(ch, "ord");
    const cls = CHAR_CLASS[ch];
    if (cls) return this.insertSymbol(CHAR_LATEX[ch] ?? ch, cls);
    if (ch === "∞") return this.insertSymbol("\\infty", "ord");
    if (ch === "'") return this.insertSymbol("'", "ord");
    // any other character (π, α, ∫ typed or pasted): a symbol of its own
    if (ch.trim()) this.insertSymbol(ch, "ord");
  }

  /** Types a string, character by character. */
  typeText(text: string): void {
    this.batch(() => {
      for (const ch of text) this.type(ch);
    });
  }

  /** Enter: finishes a command being typed; in a matrix, goes to the cell below, adding a line at the last one. */
  enter(): void {
    const command = this.commandBeingTyped();
    if (command) return this.change(() => this.finishCommand(command));
    const at = this.matrixAtCursor();
    if (!at) return;
    this.change(() => {
      if (at.i === at.matrix.cells.length - 1) this.insertMatrixRow(at.matrix, at.i + 1);
      const target = at.matrix.cells[at.i + 1][at.j];
      this.cursor = { row: target, offset: target.children.length };
      this.anchor = null;
    });
  }

  /** Finishes the command being typed with this name (what a command menu does). */
  completeCommand(name: string): void {
    const command = this.commandBeingTyped();
    if (!command) return;
    this.change(() => {
      command.name = name.replace(/^\\/, "");
      this.finishCommand(command);
    });
  }

  /** Inserts one symbol at the cursor (replacing the selection): `x`, `\alpha`, `\pm` (class `bin`)… */
  insertSymbol(latex: string, cls?: AtomClass): void {
    const c = cls ?? (latex.startsWith("\\") ? classOfCommand(latex.slice(1)) : (CHAR_CLASS[latex] ?? "ord"));
    this.change(() => this.replaceSelectionWith([{ kind: "symbol", latex, cls: c }]));
  }

  /** Inserts a structure by name (`frac`, `sqrt`, `sum`, `pmatrix`, `vec`, `sup`…; see `TEMPLATES`). */
  insert(name: string): void {
    if (name === "sup" || name === "sub") return this.script(name);
    const snippet = TEMPLATES[name];
    if (!snippet) throw new Error(`mathra-editor: no template named ${name}`);
    this.change(() => this.insertSpecs(readLatex(snippet, { autoDelimiters: true })));
  }

  /** Inserts LaTeX at the cursor. In a snippet, `#0` is where the selection goes and `#?` an empty
   * place (when it is a whole argument); the cursor goes to the first empty place inserted, or after
   * what was inserted. With `autoDelimiters`, `\\left…\\right` stays plain around small things. */
  insertLatex(latex: string, options: { autoDelimiters?: boolean } = {}): void {
    this.change(() => {
      if (this.inText() && !/[#\\{}^_]/.test(latex)) {
        this.replaceSelectionWith([...latex].map((ch) => ({ kind: "char" as const, ch })));
        return;
      }
      this.insertSpecs(readLatex(latex, options));
    });
  }

  // ---------- moving ----------

  /** Right: into the next structure, out at the end of a row. With `extend`, grows the selection by whole atoms. */
  moveRight(extend = false): void {
    this.change(() => (extend ? this.extend(+1) : this.step(+1)));
  }

  moveLeft(extend = false): void {
    this.change(() => (extend ? this.extend(-1) : this.step(-1)));
  }

  /** Up: from a denominator to its numerator, an index to the exponent, a cell to the one above. */
  moveUp(): void {
    this.change(() => this.vertical(-1));
  }

  moveDown(): void {
    this.change(() => this.vertical(+1));
  }

  home(extend = false): void {
    this.change(() => {
      if (extend) this.anchor ??= { ...this.cursor };
      else this.anchor = null;
      this.cursor = { row: this.cursor.row, offset: 0 };
      this.dropEmptySelection();
    });
  }

  end(extend = false): void {
    this.change(() => {
      if (extend) this.anchor ??= { ...this.cursor };
      else this.anchor = null;
      this.cursor = { row: this.cursor.row, offset: this.cursor.row.children.length };
      this.dropEmptySelection();
    });
  }

  /** Tab: the next empty place, else the next cell, else the end of the formula. Shift+Tab: back. */
  tab(back = false): void {
    this.change(() => {
      this.anchor = null;
      const rows = this.rows();
      const here = rows.indexOf(this.cursor.row);
      const order = back ? rows.slice(0, here).reverse() : rows.slice(here + 1);
      const empty = order.find((r) => r.children.length === 0);
      if (empty) return void (this.cursor = { row: empty, offset: 0 });
      const at = this.matrixAtCursor();
      if (at) {
        const flat = at.matrix.cells.flat();
        const k = flat.indexOf(at.matrix.cells[at.i][at.j]) + (back ? -1 : 1);
        if (k >= 0 && k < flat.length) return void (this.cursor = { row: flat[k], offset: back ? flat[k].children.length : 0 });
      }
      this.cursor = back ? { row: this.root, offset: 0 } : { row: this.root, offset: this.root.children.length };
    });
  }

  /** Puts the cursor somewhere, as a click does; with `extend`, keeps the selection's other end (a drag, Shift+click). */
  setCursor(cursor: Cursor, extend = false): void {
    this.change(() => {
      if (extend) this.anchor ??= { ...this.cursor };
      else this.anchor = null;
      this.cursor = { row: cursor.row, offset: Math.max(0, Math.min(cursor.offset, cursor.row.children.length)) };
      this.dropEmptySelection();
    });
  }

  /** Selects everything. */
  selectAll(): void {
    this.change(() => {
      this.anchor = { row: this.root, offset: 0 };
      this.cursor = { row: this.root, offset: this.root.children.length };
      this.dropEmptySelection();
    });
  }

  /** Selects the atoms `start` to `end` of a row. */
  select(row: Row, start: number, end: number): void {
    this.change(() => {
      this.anchor = { row, offset: start };
      this.cursor = { row, offset: end };
      this.dropEmptySelection();
    });
  }

  // ---------- deleting ----------

  /** Backspace: deletes the selection, or the atom before the cursor; a structure with something
   * inside is entered from its end; at the start of a place, the structure goes if it is empty. */
  backspace(): void {
    this.change(() => {
      if (this.deleteSelection()) return;
      const { row, offset } = this.cursor;
      if (offset > 0) {
        const atom = row.children[offset - 1];
        if (atom.kind === "command" && atom.name.length > 0) return void (atom.name = atom.name.slice(0, -1));
        const rows = childRows(atom);
        const filled = [...rows].reverse().find((r) => r.children.length > 0);
        if (filled) return void (this.cursor = { row: filled, offset: filled.children.length });
        return this.removeAt(row, offset - 1);
      }
      this.leaveOrRemoveOwner(row, -1);
    });
  }

  /** Delete: the selection, or the atom after the cursor (a structure with something inside is entered from its start). */
  deleteForward(): void {
    this.change(() => {
      if (this.deleteSelection()) return;
      const { row, offset } = this.cursor;
      if (offset < row.children.length) {
        const atom = row.children[offset];
        const filled = childRows(atom).find((r) => r.children.length > 0);
        if (filled) return void (this.cursor = { row: filled, offset: 0 });
        return this.removeAt(row, offset);
      }
      this.leaveOrRemoveOwner(row, +1);
    });
  }

  // ---------- clipboard ----------

  /** The selection as plain LaTeX, for the clipboard. */
  copy(): string {
    return this.selectedLatex();
  }

  /** The selection as plain LaTeX, removed from the formula. */
  cut(): string {
    const latex = this.selectedLatex();
    if (latex) this.change(() => void this.deleteSelection());
    return latex;
  }

  /** Pastes LaTeX (or plain text inside `\text{…}`) at the cursor, replacing the selection. */
  paste(text: string): void {
    const latex = text.trim().replace(/^\$\$?([\s\S]*?)\$?\$$/, "$1").replace(/^\\\[([\s\S]*)\\\]$/, "$1").replace(/^\\\(([\s\S]*)\\\)$/, "$1");
    this.change(() => {
      if (this.inText()) return this.replaceSelectionWith([...text].map((ch) => ({ kind: "char" as const, ch })));
      this.insertSpecs(readLatex(latex), { cursorAfter: true });
    });
  }

  // ---------- matrices ----------

  /** Adds a line to the matrix around the cursor, below the cursor's line (above with `before`). */
  addRow(before = false): void {
    const at = this.matrixAtCursor();
    if (!at) return;
    this.change(() => {
      const i = before ? at.i : at.i + 1;
      this.insertMatrixRow(at.matrix, i);
      this.cursor = { row: at.matrix.cells[i][at.j], offset: 0 };
      this.anchor = null;
    });
  }

  /** Adds a column to the matrix around the cursor, right of the cursor's column (left with `before`). */
  addColumn(before = false): void {
    const at = this.matrixAtCursor();
    if (!at) return;
    this.change(() => {
      const j = before ? at.j : at.j + 1;
      this.insertMatrixColumn(at.matrix, j);
      this.cursor = { row: at.matrix.cells[at.i][j], offset: 0 };
      this.anchor = null;
    });
  }

  /** Removes the cursor's line from the matrix around it (the matrix goes with its last line). */
  removeRow(): void {
    const at = this.matrixAtCursor();
    if (!at) return;
    this.change(() => {
      const m = at.matrix;
      if (m.cells.length === 1) return this.removeAtom(m);
      m.cells.splice(at.i, 1);
      const i = Math.min(at.i, m.cells.length - 1);
      this.cursor = { row: m.cells[i][at.j], offset: 0 };
      this.anchor = null;
    });
  }

  /** Removes the cursor's column from the matrix around it (the matrix goes with its last column). */
  removeColumn(): void {
    const at = this.matrixAtCursor();
    if (!at) return;
    this.change(() => {
      const m = at.matrix;
      if (m.cells[0].length === 1) return this.removeAtom(m);
      for (const line of m.cells) line.splice(at.j, 1);
      const j = Math.min(at.j, m.cells[0].length - 1);
      this.cursor = { row: m.cells[at.i][j], offset: 0 };
      this.anchor = null;
    });
  }

  // ---------- undo ----------

  canUndo(): boolean {
    return this.undoStack.length > 0;
  }

  canRedo(): boolean {
    return this.redoStack.length > 0;
  }

  undo(): void {
    const state = this.undoStack.pop();
    if (state === undefined) return;
    this.redoStack.push(this.snapshot());
    this.restore(state);
    this.emit("content");
  }

  redo(): void {
    const state = this.redoStack.pop();
    if (state === undefined) return;
    this.undoStack.push(this.snapshot());
    this.restore(state);
    this.emit("content");
  }

  /** Runs several commands as one change: one undo step, one notification. */
  batch(f: () => void): void {
    this.change(() => {
      this.batching++;
      try {
        f();
      } finally {
        this.batching--;
      }
    });
  }

  // ---------- internals: changes ----------

  private id(): string {
    return `n${this.next++}`;
  }

  private row(parent: Atom | null): Row {
    return { kind: "row", id: this.id(), children: [], parent };
  }

  private change(f: () => void): void {
    if (this.batching) return f();
    const before = this.snapshot();
    const content = this.contentKey();
    f();
    const after = this.snapshot();
    if (after === before) return;
    const contentChanged = this.contentKey() !== content;
    if (contentChanged) {
      if (this.undoStack[this.undoStack.length - 1] !== before) this.undoStack.push(before);
      if (this.undoStack.length > 500) this.undoStack.shift();
      this.redoStack = [];
    }
    this.emit(contentChanged ? "content" : "cursor");
  }

  /** The formula, without the cursor. */
  private contentKey(): string {
    return JSON.stringify(this.listSpec(this.root.children));
  }

  private emit(kind: ChangeKind): void {
    for (const l of this.listeners) l(kind);
  }

  // ---------- internals: building ----------

  /** Makes atoms from specs, into `row` at `at`; `#0` takes `selected`. Returns the atoms made. */
  private adoptInto(row: Row, specs: AtomSpec[], selected: Atom[], at = row.children.length): Atom[] {
    const made: Atom[] = [];
    for (const spec of specs) {
      if (spec.kind === "selection") {
        for (const a of selected) {
          a.parent = row;
          made.push(a);
        }
        selected = [];
        continue;
      }
      made.push(this.adopt(spec, row, selected));
    }
    row.children.splice(at, 0, ...made);
    return made;
  }

  private adopt(spec: AtomSpec, parent: Row, selected: Atom[]): Atom {
    const base = { id: this.id(), parent };
    const rowOf = (owner: Atom, specs: AtomSpec[] | null): Row | null => {
      if (!specs) return null;
      const r = this.row(owner);
      this.adoptInto(r, specs, selected);
      return r;
    };
    switch (spec.kind) {
      case "symbol":
        return { ...base, kind: "symbol", latex: spec.latex, cls: spec.cls };
      case "char":
        return { ...base, kind: "char", ch: spec.ch };
      case "raw":
        return { ...base, kind: "raw", latex: spec.latex };
      case "command":
        return { ...base, kind: "command", name: spec.name };
      case "text": {
        const a: TextAtom = { ...base, kind: "text", cmd: spec.cmd, body: null! };
        // the selection inside text becomes its LaTeX, as characters
        const specs = spec.body.flatMap((s): AtomSpec[] => (s.kind === "selection" ? [...this.listLatex(selected, false)].map((ch) => ({ kind: "char" as const, ch })) : [s]));
        if (spec.body.some((s) => s.kind === "selection")) selected = [];
        a.body = rowOf(a, specs)!;
        return a;
      }
      case "frac": {
        const a: Fraction = { ...base, kind: "frac", cmd: spec.cmd, num: null!, den: null! };
        a.num = rowOf(a, spec.num)!;
        a.den = rowOf(a, spec.den)!;
        return a;
      }
      case "sqrt": {
        const a: Root = { ...base, kind: "sqrt", index: null, body: null! };
        a.index = rowOf(a, spec.index);
        a.body = rowOf(a, spec.body)!;
        return a;
      }
      case "scripts": {
        const a: Scripts = { ...base, kind: "scripts", sup: null, sub: null };
        a.sup = rowOf(a, spec.sup);
        a.sub = rowOf(a, spec.sub);
        return a;
      }
      case "delim": {
        const a: Delimited = { ...base, kind: "delim", left: spec.left, right: spec.right, body: null!, auto: spec.auto ?? false };
        a.body = rowOf(a, spec.body)!;
        return a;
      }
      case "bigop": {
        const a: LargeOperator = { ...base, kind: "bigop", op: spec.op, sub: null, sup: null };
        a.sub = rowOf(a, spec.sub);
        a.sup = rowOf(a, spec.sup);
        return a;
      }
      case "accent":
      case "style": {
        const a = { ...base, kind: spec.kind, cmd: spec.cmd, body: null! } as Accent | Styled;
        a.body = rowOf(a, spec.body)!;
        return a;
      }
      case "matrix": {
        const a: Matrix = { ...base, kind: "matrix", env: spec.env, cells: [] };
        a.cells = spec.cells.map((line) => line.map((cell) => rowOf(a, cell)!));
        return a;
      }
      case "selection":
        throw new Error("mathra-editor: #0 is placed by adoptInto");
    }
  }

  /** Inserts specs at the cursor, the selection going where `#0` is (or replaced, without `#0`). */
  private insertSpecs(specs: AtomSpec[], options: { cursorAfter?: boolean } = {}): void {
    const sel = this.selection();
    let selected: Atom[] = [];
    if (sel) {
      selected = sel.row.children.splice(sel.start, sel.end - sel.start);
      this.cursor = { row: sel.row, offset: sel.start };
      this.anchor = null;
    }
    const { row, offset } = this.cursor;
    // a lone exponent or index joins the one before it
    if (specs.length === 1 && specs[0].kind === "scripts" && !selected.length) {
      const before = row.children[offset - 1];
      const s = specs[0];
      if (before && before.kind === "scripts" && (!s.sup || !before.sup) && (!s.sub || !before.sub)) {
        const made = this.adopt(s, row, []) as Scripts;
        for (const which of ["sup", "sub"] as const) {
          const r = made[which];
          if (r) {
            r.parent = before;
            before[which] = r;
          }
        }
        const first = (made.sup ?? made.sub)!;
        this.cursor = { row: first, offset: first.children.length };
        return;
      }
    }
    const made = this.adoptInto(row, specs, selected, offset);
    this.cursor = { row, offset: offset + made.length };
    if (options.cursorAfter) return;
    // the first empty place inserted
    for (const a of made) {
      const empty = rowsOf(a).find((r) => r.children.length === 0);
      if (empty) {
        this.cursor = { row: empty, offset: 0 };
        return;
      }
    }
  }

  private replaceSelectionWith(specs: AtomSpec[]): void {
    this.deleteSelection();
    const { row, offset } = this.cursor;
    const made = this.adoptInto(row, specs, [], offset);
    this.cursor = { row, offset: offset + made.length };
  }

  private finishCommand(command: Command): void {
    const row = command.parent!;
    const at = row.children.indexOf(command);
    row.children.splice(at, 1);
    this.cursor = { row, offset: at };
    const name = command.name;
    if (!name) return;
    const snippet = TEMPLATES[name];
    if (snippet) return this.insertSpecs(readLatex(snippet, { autoDelimiters: true }));
    // a large operator typed alone: with its limits, as the palette inserts it
    if (LOWER_LIMIT_ONLY.has(name)) return this.insertSpecs(readLatex(`\\${name}_{#?}`));
    this.insertSpecs(readLatex(`\\${name}`), { cursorAfter: true });
  }

  private script(which: ScriptTemplate): void {
    this.change(() => this.addScript(which));
  }

  private addScript(which: ScriptTemplate): void {
    this.deleteSelection();
    const { row, offset } = this.cursor;
    const before = row.children[offset - 1];
    // a large operator takes it as its limit
    if (before && (before.kind === "scripts" || before.kind === "bigop")) {
      const existing = before[which];
      if (existing) return void (this.cursor = { row: existing, offset: existing.children.length });
      const r = this.row(before);
      before[which] = r;
      this.cursor = { row: r, offset: 0 };
      return;
    }
    this.insertSpecs([{ kind: "scripts", sup: which === "sup" ? [] : null, sub: which === "sub" ? [] : null }]);
  }

  /** `/`: the operand before the cursor (back to an operator, a relation or an opening) becomes the numerator. */
  private fractionOfOperand(): void {
    if (this.selection()) return this.insertSpecs(readLatex("\\frac{#0}{#?}"));
    const { row, offset } = this.cursor;
    let start = offset;
    let depth = 0;
    while (start > 0) {
      const a = row.children[start - 1];
      if (a.kind === "symbol" && a.cls === "close") depth++;
      else if (a.kind === "symbol" && a.cls === "open") {
        if (depth === 0) break;
        depth--;
      } else if (depth === 0 && (a.kind === "command" || a.kind === "bigop" || (a.kind === "symbol" && a.cls !== "ord"))) break;
      start--;
    }
    this.anchor = start < offset ? { row, offset: start } : null;
    this.insertSpecs(readLatex(start < offset ? "\\frac{#0}{#?}" : "\\frac{#?}{#?}"));
  }

  /** `)` typed at the end of a `(`…`)` (or at the end of what ends it, such as a denominator): leaves it. */
  private leaveDelimiter(closer: string): boolean {
    if (this.anchor) return false;
    let row = this.cursor.row;
    let offset = this.cursor.offset;
    while (row.parent) {
      if (offset !== row.children.length) return false;
      const owner = row.parent;
      const outer = owner.parent!;
      const at = outer.children.indexOf(owner);
      if (owner.kind === "delim" && owner.right === closer) {
        this.change(() => (this.cursor = { row: outer, offset: at + 1 }));
        return true;
      }
      // the last place of its structure, and the structure last in its row
      const rows = childRows(owner);
      if (rows[rows.length - 1] !== row) return false;
      row = outer;
      offset = at + 1;
    }
    return false;
  }

  private nextCell(): void {
    const at = this.matrixAtCursor();
    if (!at) return;
    this.change(() => {
      const m = at.matrix;
      if (at.j === m.cells[at.i].length - 1) this.insertMatrixColumn(m, at.j + 1);
      const target = m.cells[at.i][at.j + 1];
      this.cursor = { row: target, offset: target.children.length };
      this.anchor = null;
    });
  }

  private insertMatrixRow(m: Matrix, i: number): void {
    const width = m.cells[0]?.length ?? 1;
    m.cells.splice(i, 0, Array.from({ length: width }, () => this.row(m)));
  }

  private insertMatrixColumn(m: Matrix, j: number): void {
    for (const line of m.cells) line.splice(j, 0, this.row(m));
  }

  // ---------- internals: moving and deleting ----------

  private step(side: 1 | -1): void {
    const sel = this.selection();
    if (sel) {
      this.anchor = null;
      this.cursor = { row: sel.row, offset: side > 0 ? sel.end : sel.start };
      return;
    }
    this.anchor = null;
    const { row, offset } = this.cursor;
    const next = row.children[side > 0 ? offset : offset - 1];
    if (next) {
      const rows = childRows(next);
      if (rows.length) {
        const r = side > 0 ? rows[0] : rows[rows.length - 1];
        this.cursor = { row: r, offset: side > 0 ? 0 : r.children.length };
      } else this.cursor = { row, offset: offset + side };
      return;
    }
    this.leave(row, side);
  }

  /** Shift+arrow: the cursor moves over whole atoms of its row; at the row's end, the selection takes the structure around it. */
  private extend(side: 1 | -1): void {
    this.anchor ??= { ...this.cursor };
    const { row, offset } = this.cursor;
    const target = offset + side;
    if (target >= 0 && target <= row.children.length) this.cursor = { row, offset: target };
    else if (row.parent) {
      const owner = row.parent;
      const outer = owner.parent!;
      const at = outer.children.indexOf(owner);
      // the selection, lifted to this row, takes the whole structure
      this.cursor = { row: outer, offset: side > 0 ? at + 1 : at };
    }
    this.dropEmptySelection();
  }

  private dropEmptySelection(): void {
    if (this.anchor && !this.selection()) this.anchor = null;
  }

  /** Leaves `row` to the next place on that side: the next row of the same structure, or the row around it. */
  private leave(row: Row, side: 1 | -1): void {
    const owner = row.parent;
    if (!owner) return;
    const rows = childRows(owner);
    const i = rows.indexOf(row) + side;
    if (i >= 0 && i < rows.length) {
      this.cursor = { row: rows[i], offset: side > 0 ? 0 : rows[i].children.length };
      return;
    }
    const outer = owner.parent!;
    const at = outer.children.indexOf(owner);
    this.cursor = { row: outer, offset: side > 0 ? at + 1 : at };
  }

  private vertical(side: 1 | -1): void {
    this.anchor = null;
    for (let row: Row = this.cursor.row; row.parent; row = row.parent.parent!) {
      const owner = row.parent;
      if (owner.kind === "matrix") {
        for (let i = 0; i < owner.cells.length; i++) {
          const j = owner.cells[i].indexOf(row);
          const line = owner.cells[i + side];
          if (j >= 0 && line) {
            this.cursor = { row: line[j], offset: line[j].children.length };
            return;
          }
        }
        continue;
      }
      const order = verticalRows(owner);
      const i = order.indexOf(row);
      const target = order[i + side];
      if (i >= 0 && target) {
        this.cursor = { row: target, offset: side > 0 ? 0 : target.children.length };
        return;
      }
    }
  }

  private deleteSelection(): boolean {
    const sel = this.selection();
    this.anchor = null;
    if (!sel) return false;
    sel.row.children.splice(sel.start, sel.end - sel.start);
    this.cursor = { row: sel.row, offset: sel.start };
    return true;
  }

  private removeAt(row: Row, i: number): void {
    row.children.splice(i, 1);
    this.cursor = { row, offset: i };
  }

  private removeAtom(atom: Atom): void {
    const row = atom.parent!;
    this.removeAt(row, row.children.indexOf(atom));
  }

  /** At the edge of a place: an empty exponent, index or root index goes alone; an empty structure goes whole; else the cursor leaves. */
  private leaveOrRemoveOwner(row: Row, side: 1 | -1): void {
    const owner = row.parent;
    if (!owner) return;
    const outer = owner.parent!;
    const at = outer.children.indexOf(owner);
    if (row.children.length === 0 && (owner.kind === "scripts" || owner.kind === "bigop") && (owner.sup === row || owner.sub === row)) {
      if (owner.sup === row) owner.sup = null;
      if (owner.sub === row) owner.sub = null;
      if (owner.kind === "scripts" && !owner.sup && !owner.sub) outer.children.splice(at, 1);
      this.cursor = { row: outer, offset: owner.kind === "scripts" && !owner.sup && !owner.sub ? at : at + 1 };
      return;
    }
    if (row.children.length === 0 && owner.kind === "sqrt" && owner.index === row) {
      owner.index = null;
      this.cursor = { row: owner.body, offset: 0 };
      return;
    }
    if (childRows(owner).every((r) => r.children.length === 0)) {
      outer.children.splice(at, 1);
      this.cursor = { row: outer, offset: at };
      return;
    }
    // a delimiter or a style around something: Backspace at its start takes it off, keeping what it holds
    if (side < 0 && row.children.length && (owner.kind === "delim" || owner.kind === "style" || owner.kind === "accent" || owner.kind === "text") && childRows(owner).length === 1) {
      const kept = row.children.splice(0);
      outer.children.splice(at, 1, ...kept);
      for (const a of kept) a.parent = outer;
      this.cursor = { row: outer, offset: at };
      return;
    }
    this.cursor = { row: outer, offset: side > 0 ? at + 1 : at };
  }

  /** From the root to the cursor's row: each level's row and the span the cursor takes in it. */
  private chain(c: Cursor): { row: Row; start: number; end: number }[] {
    const levels: { row: Row; start: number; end: number }[] = [{ row: c.row, start: c.offset, end: c.offset }];
    for (let r = c.row; r.parent; r = r.parent.parent!) {
      const owner = r.parent;
      const outer = owner.parent!;
      const i = outer.children.indexOf(owner);
      levels.unshift({ row: outer, start: i, end: i + 1 });
    }
    return levels;
  }

  // ---------- internals: LaTeX ----------

  private rowLatex(row: Row, marked: boolean): string {
    if (row.children.length === 0) return marked ? `\\htmlData{m=${row.id},empty=1}{\\square}` : "";
    const inner = this.listLatex(row.children, marked);
    return marked ? `\\htmlData{m=${row.id}}{${inner}}` : inner;
  }

  private listLatex(atoms: Atom[], marked: boolean): string {
    let out = "";
    for (let i = 0; i < atoms.length; i++) {
      // where the locale rewrites a number's point (3.14 is 3,14 in pt-BR), the number is one group:
      // marks between its digits would keep mathra from reading it as a number
      if (marked) {
        const run = markedRun(atoms, i, this.decimalComma);
        if (run) {
          out = join(out, `\\htmlData{m=${atoms[i].id}n}{${run.map((a) => (a as SymbolAtom).latex).join("")}}`);
          i += run.length - 1;
          continue;
        }
      }
      const atom = atoms[i];
      // \underbrace{…}_{n}: a mark around it would turn the label below into an index
      const bare = marked && atom.kind === "accent" && /^\\(under|over)brace$/.test(atom.cmd) && atoms[i + 1]?.kind === "scripts";
      out = join(out, bare ? `${atom.cmd}{${this.rowLatex(atom.body, true)}}` : this.atomLatex(atom, marked, atoms[i - 1], atoms[i + 1]));
    }
    return out;
  }

  private atomLatex(atom: Atom, marked: boolean, before?: Atom, after?: Atom): string {
    const mark = (s: string) => (marked ? `\\htmlData{m=${atom.id}}{${s}}` : s);
    const r = (row: Row) => this.rowLatex(row, marked);
    switch (atom.kind) {
      case "symbol": {
        if (!marked) return atom.latex;
        let cls = atom.cls;
        // a decimal comma, where the locale reads one: part of the number, as mathra reads it
        if (atom.latex === "," && this.decimalComma && isDigit(before) && isDigit(after)) cls = "ord";
        const wrapper = CLASS_WRAPPER[cls];
        const m = mark(atom.latex);
        return wrapper ? `${wrapper}{${m}}` : m;
      }
      case "char":
        // marked, each character is its own \text, which mathra draws as the whole word
        return marked ? mark(`\\text{${atom.ch === " " ? "\\ " : escapeText(atom.ch)}}`) : escapeText(atom.ch);
      case "text":
        if (!marked) return `${atom.cmd}{${atom.body.children.map((c) => (c.kind === "char" ? escapeText(c.ch) : "")).join("")}}`;
        return mark(r(atom.body).replace(/\\text\{/g, `${atom.cmd}{`));
      case "raw":
        return marked ? mark(atom.latex) : atom.latex;
      case "command":
        return marked ? mark(`\\mathtt{\\backslash ${atom.name}}`) : "";
      case "frac": {
        const m = mark(`${atom.cmd}{${r(atom.num)}}{${r(atom.den)}}`);
        // \overset{?}{=} is a relation, as its base is: the mark keeps that class
        const base = /^\\(overset|underset|stackrel)$/.test(atom.cmd) && atom.den.children.length === 1 ? atom.den.children[0] : null;
        const wrapper = marked && base?.kind === "symbol" ? CLASS_WRAPPER[base.cls] : null;
        return wrapper ? `${wrapper}{${m}}` : m;
      }
      case "sqrt":
        return mark(atom.index ? `\\sqrt[${r(atom.index)}]{${r(atom.body)}}` : `\\sqrt{${r(atom.body)}}`);
      case "accent":
      case "style":
        // \color{c} is a switch: it and what it colors go in a group of their own
        if (atom.cmd.startsWith("\\color{")) return marked ? mark(`${atom.cmd}${r(atom.body)}`) : `{${atom.cmd}${r(atom.body)}}`;
        return mark(`${atom.cmd}{${r(atom.body)}}`);
      case "bigop":
        return mark(`${atom.op}${atom.sub ? `_{${r(atom.sub)}}` : ""}${atom.sup ? `^{${r(atom.sup)}}` : ""}`);
      case "scripts": {
        // on the atom before it; at the start of a row, or after another script, on an empty base
        const base = !before || before.kind === "scripts" ? "{}" : "";
        return `${base}${atom.sup ? `^{${r(atom.sup)}}` : ""}${atom.sub ? `_{${r(atom.sub)}}` : ""}`;
      }
      case "delim": {
        const left = DELIMITERS[atom.left] ?? { open: atom.left, close: atom.left };
        const right = DELIMITERS[atom.right] ?? { open: atom.right, close: atom.right };
        if (!atom.auto || isTall(atom.body)) return mark(`\\left${atom.left}${spaceAfter(atom.left)}${r(atom.body)}\\right${atom.right}`);
        // small: plain delimiters, which keep TeX's spacing (f(x), \sin(x))
        if (!marked) return `${left.open}${spaceAfter(left.open)}${r(atom.body)}${right.close}`;
        const side = (latex: string, cls: "open" | "close", suffix: string) => (latex ? `\\math${cls}{\\htmlData{m=${atom.id}${suffix}}{${latex}}}` : "");
        return `${side(left.open, "open", "l")}${r(atom.body)}${side(right.close, "close", "r")}`;
      }
      case "matrix": {
        const body = atom.cells
          .map((line) => line.map((cell) => r(cell)).join(" & "))
          .join(" \\\\ ")
          .replace(/ {2,}/g, " ");
        return mark(`\\begin{${atom.env}}${body}\\end{${atom.env}}`);
      }
    }
  }

  // ---------- internals: undo ----------

  private listSpec(atoms: Atom[]): AtomSpec[] {
    return atoms.map((a) => this.spec(a));
  }

  private spec(a: Atom): AtomSpec {
    const l = (r: Row | null) => (r ? this.listSpec(r.children) : null);
    switch (a.kind) {
      case "symbol":
        return { kind: "symbol", latex: a.latex, cls: a.cls };
      case "char":
        return { kind: "char", ch: a.ch };
      case "raw":
        return { kind: "raw", latex: a.latex };
      case "command":
        return { kind: "command", name: a.name };
      case "text":
        return { kind: "text", cmd: a.cmd, body: l(a.body)! };
      case "frac":
        return { kind: "frac", cmd: a.cmd, num: l(a.num)!, den: l(a.den)! };
      case "sqrt":
        return { kind: "sqrt", index: l(a.index), body: l(a.body)! };
      case "scripts":
        return { kind: "scripts", sup: l(a.sup), sub: l(a.sub) };
      case "delim":
        return a.auto ? { kind: "delim", left: a.left, right: a.right, body: l(a.body)!, auto: true } : { kind: "delim", left: a.left, right: a.right, body: l(a.body)! };
      case "bigop":
        return { kind: "bigop", op: a.op, sub: l(a.sub), sup: l(a.sup) };
      case "accent":
      case "style":
        return { kind: a.kind, cmd: a.cmd, body: l(a.body)! };
      case "matrix":
        return { kind: "matrix", env: a.env, cells: a.cells.map((line) => line.map((c) => l(c)!)) };
    }
  }

  private snapshot(): string {
    const s: Snapshot = {
      root: this.listSpec(this.root.children),
      cursor: this.path(this.cursor.row),
      offset: this.cursor.offset,
      anchor: this.anchor ? this.path(this.anchor.row) : null,
      anchorOffset: this.anchor?.offset ?? 0,
    };
    return JSON.stringify(s);
  }

  private restore(state: string): void {
    const s = JSON.parse(state) as Snapshot;
    this.root = this.row(null);
    this.adoptInto(this.root, s.root, []);
    const find = (path: [number, number][]) => {
      let row = this.root;
      for (const [i, j] of path) row = childRows(row.children[i])[j];
      return row;
    };
    this.cursor = { row: find(s.cursor), offset: s.offset };
    this.anchor = s.anchor ? { row: find(s.anchor), offset: s.anchorOffset } : null;
  }

  /** How to reach `row` from the root: for each step, the atom's index and the row's index in it. */
  private path(row: Row): [number, number][] {
    const steps: [number, number][] = [];
    for (let r = row; r.parent; r = r.parent.parent!) {
      const owner = r.parent;
      steps.unshift([owner.parent!.children.indexOf(owner), childRows(owner).indexOf(r)]);
    }
    return steps;
  }
}

/** An atom's rows, in the order the cursor visits them from left to right. */
export function childRows(atom: Atom): Row[] {
  switch (atom.kind) {
    case "frac":
      return [atom.num, atom.den];
    case "sqrt":
      return atom.index ? [atom.index, atom.body] : [atom.body];
    case "scripts":
      return [atom.sup, atom.sub].filter((r): r is Row => r !== null);
    case "bigop":
      return [atom.sub, atom.sup].filter((r): r is Row => r !== null);
    case "delim":
    case "accent":
    case "style":
    case "text":
      return [atom.body];
    case "matrix":
      return atom.cells.flat();
    default:
      return [];
  }
}

/** All rows inside an atom, deep, in document order. */
function rowsOf(atom: Atom): Row[] {
  return childRows(atom).flatMap((r) => [r, ...r.children.flatMap(rowsOf)]);
}

/** An atom's rows from top to bottom, for up and down. */
function verticalRows(atom: Atom): Row[] {
  switch (atom.kind) {
    case "frac":
      return [atom.num, atom.den];
    case "scripts":
    case "bigop":
      return [atom.sup, atom.sub].filter((r): r is Row => r !== null);
    case "sqrt":
      return atom.index ? [atom.index, atom.body] : [];
    default:
      return [];
  }
}

/** Something in the row is taller than a line: delimiters around it grow. */
function isTall(row: Row): boolean {
  return row.children.some((a) => a.kind === "frac" || a.kind === "bigop" || a.kind === "matrix" || (a.kind === "delim" && isTall(a.body)) || (a.kind === "raw" && /\\(begin|frac|dfrac|sum|int|prod)\b/.test(a.latex)) || ((a.kind === "sqrt" || a.kind === "style" || a.kind === "accent") && childRows(a).some(isTall)));
}

/** Atoms that must be marked as one group, from `atoms[i]`: primes (`''` is one double prime), and a
 * number with a script after it (`10^x`: the exponent is on the whole number) or, where the locale
 * rewrites its point (`decimalComma`), with a point. None if `atoms[i]` starts none. */
export function markedRun(atoms: Atom[], i: number, decimalComma: boolean): Atom[] | null {
  const prime = (a: Atom | undefined) => !!a && a.kind === "symbol" && a.latex === "'";
  if (prime(atoms[i]) && !prime(atoms[i - 1])) {
    let j = i;
    while (prime(atoms[j + 1])) j++;
    return j > i ? atoms.slice(i, j + 1) : null;
  }
  const run = numberRun(atoms, i);
  if (!run || run.length < 2) return null;
  const scripted = atoms[i + run.length]?.kind === "scripts";
  const point = decimalComma && run.some((a) => a.kind === "symbol" && a.latex === ".");
  return scripted || point ? run : null;
}

/** The number starting at `atoms[i]` (digits, and points or commas between digits); none if `atoms[i]` does not start one. */
export function numberRun(atoms: Atom[], i: number): Atom[] | null {
  if (!isDigit(atoms[i]) || isDigit(atoms[i - 1]) || (isSeparator(atoms[i - 1]) && isDigit(atoms[i - 2]))) return null;
  let j = i;
  while (isDigit(atoms[j + 1]) || (isSeparator(atoms[j + 1]) && isDigit(atoms[j + 2]))) j++;
  return atoms.slice(i, j + 1);
}

function isSeparator(a: Atom | undefined): boolean {
  return !!a && a.kind === "symbol" && (a.latex === "." || a.latex === ",");
}

function isDigit(a: Atom | undefined): boolean {
  return !!a && a.kind === "symbol" && /^[0-9]$/.test(a.latex);
}

/** A delimiter command needs a space before a letter (`\langle x`). */
function spaceAfter(d: string): string {
  return /\\[A-Za-z]+$/.test(d) ? " " : "";
}

/** Two pieces of LaTeX one after the other, with a space only where a control word would swallow a letter. */
function join(a: string, b: string): string {
  if (!a) return b;
  return /\\[A-Za-z]+$/.test(a) && /^[A-Za-z]/.test(b) ? `${a} ${b}` : a + b;
}
