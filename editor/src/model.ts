// The formula being edited: a tree of rows and atoms, a cursor, and the commands that change them.
// No DOM and no mathra here: `toLatex` writes the LaTeX that mathra draws, with each atom and each
// row marked (`\htmlData{m=<id>}{…}`) so the boxes mathra returns tell where everything sits.

/** TeX's atom classes, which set the spacing around a symbol. */
export type AtomClass = "ord" | "bin" | "rel" | "open" | "close" | "punct" | "op";

/** A horizontal list: the whole formula, a numerator, an exponent, a root's body… */
export interface Row {
  kind: "row";
  id: string;
  children: Atom[];
  /** The atom this row belongs to; none for the formula itself. */
  parent: Atom | null;
}

interface AtomBase {
  id: string;
  /** The row this atom is in. */
  parent: Row | null;
}

/** One symbol: a letter, a digit, an operator, `\alpha`, `\sin`… */
export interface SymbolAtom extends AtomBase {
  kind: "symbol";
  latex: string;
  cls: AtomClass;
}

export interface Fraction extends AtomBase {
  kind: "frac";
  num: Row;
  den: Row;
}

export interface Root extends AtomBase {
  kind: "sqrt";
  /** The index of an n-th root; none for a square root. */
  index: Row | null;
  body: Row;
}

/** An exponent, an index, or both, on the atom before it. */
export interface Scripts extends AtomBase {
  kind: "scripts";
  sup: Row | null;
  sub: Row | null;
}

/** Delimiters that grow with what they hold (`\left( … \right)`). */
export interface Delimited extends AtomBase {
  kind: "delim";
  left: string;
  right: string;
  body: Row;
}

/** A large operator with limits: `\sum`, `\prod`, `\int`, `\lim`. */
export interface LargeOperator extends AtomBase {
  kind: "bigop";
  op: string;
  sub: Row | null;
  sup: Row | null;
}

/** A command being typed after `\`, until a space or Enter turns it into what it names. */
export interface Command extends AtomBase {
  kind: "command";
  name: string;
}

export type Atom = SymbolAtom | Fraction | Root | Scripts | Delimited | LargeOperator | Command;

/** Where the cursor is: between two atoms of a row, `offset` atoms from its start. */
export interface Cursor {
  row: Row;
  offset: number;
}

/** The structures the palette and the commands insert. */
export type Template = "frac" | "sqrt" | "nroot" | "sup" | "sub" | "paren" | "brackets" | "abs" | "sum" | "prod" | "int" | "lim";

/** Symbols a `\name` turns into, with their class. Anything else stays as `\name`, an ordinary symbol. */
const SYMBOLS: Record<string, AtomClass> = {
  pm: "bin", mp: "bin", times: "bin", div: "bin", cdot: "bin", cup: "bin", cap: "bin", circ: "bin", setminus: "bin",
  le: "rel", ge: "rel", leq: "rel", geq: "rel", ne: "rel", neq: "rel", approx: "rel", equiv: "rel", sim: "rel", propto: "rel",
  in: "rel", notin: "rel", subset: "rel", subseteq: "rel", supset: "rel", to: "rel", rightarrow: "rel", leftarrow: "rel",
  Rightarrow: "rel", Leftrightarrow: "rel", mapsto: "rel", perp: "rel", parallel: "rel",
  sin: "op", cos: "op", tan: "op", sen: "op", tg: "op", log: "op", ln: "op", exp: "op", det: "op", max: "op", min: "op",
};

/** Commands that insert a structure. */
const TEMPLATES: Record<string, Template> = { frac: "frac", sqrt: "sqrt", root: "nroot", sum: "sum", prod: "prod", int: "int", lim: "lim" };

/** What a typed character is. */
const CLASS_OF_CHAR: Record<string, AtomClass> = { "+": "bin", "-": "bin", "*": "bin", "=": "rel", "<": "rel", ">": "rel", "(": "open", "[": "open", ")": "close", "]": "close", ",": "punct", ";": "punct", "!": "close" };
const LATEX_OF_CHAR: Record<string, string> = { "*": "\\cdot" };

/** Wrappers that keep a marked symbol's class: a group alone is an ordinary atom, as in TeX. */
const CLASS_WRAPPER: Record<AtomClass, string | null> = { ord: null, bin: "\\mathbin", rel: "\\mathrel", open: "\\mathopen", close: "\\mathclose", punct: "\\mathpunct", op: null };

export interface LatexOptions {
  /** Mark every atom and row with `\htmlData{m=<id>}`, and draw empty rows as `\square`: what the editor draws. Off: the plain LaTeX to hand out. */
  marked?: boolean;
}

export type ChangeListener = () => void;

/** An editor without a screen: the formula, the cursor and the commands. */
export class Editor {
  root: Row;
  cursor: Cursor;
  private next = 0;
  private listeners = new Set<ChangeListener>();
  private undoStack: string[] = [];
  private redoStack: string[] = [];

  constructor() {
    this.root = this.row(null);
    this.cursor = { row: this.root, offset: 0 };
  }

  // ---------- reading ----------

  /** The formula as LaTeX: plain to hand out, or marked to draw with mathra. */
  toLatex(options: LatexOptions = {}): string {
    const marked = options.marked ?? false;
    return this.rowLatex(this.root, marked);
  }

  /** Every row, outer before inner. */
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

  /** Called after every change of the formula or the cursor. Returns a function that removes it. */
  onChange(listener: ChangeListener): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  // ---------- typing ----------

  /** Types one character, as a keyboard does: letters, digits and operators become symbols; `/` makes
   * a fraction of what is before the cursor; `^` and `_` an exponent and an index; `\` starts a command,
   * which a space or Enter finishes. */
  type(ch: string): void {
    const command = this.typingCommand();
    if (command) {
      if (/^[A-Za-z]$/.test(ch)) return this.change(() => void (command.name += ch));
      this.change(() => this.finishCommand(command));
      if (ch === " ") return;
    }
    if (ch === "\\") return this.change(() => this.insertAtom({ kind: "command", id: this.id(), parent: null, name: "" }));
    if (ch === "/") return this.change(() => this.fractionOfOperand());
    if (ch === "^") return this.insert("sup");
    if (ch === "_") return this.insert("sub");
    if (ch === " ") return;
    if (/^[A-Za-z0-9.]$/.test(ch)) return this.insertSymbol(ch, "ord");
    const cls = CLASS_OF_CHAR[ch];
    if (cls) return this.insertSymbol(LATEX_OF_CHAR[ch] ?? ch, cls);
    if (ch === "|") return this.insertSymbol("|", "ord");
  }

  /** Types a string, character by character. */
  typeText(text: string): void {
    for (const ch of text) this.type(ch);
  }

  /** Enter: finishes a command being typed. */
  enter(): void {
    const command = this.typingCommand();
    if (command) this.change(() => this.finishCommand(command));
  }

  /** Inserts one symbol at the cursor: `x`, `\alpha`, `\pm` (class `bin`)… */
  insertSymbol(latex: string, cls: AtomClass = "ord"): void {
    this.change(() => this.insertAtom({ kind: "symbol", id: this.id(), parent: null, latex, cls }));
  }

  /** Inserts a structure at the cursor and puts the cursor in its first empty place. */
  insert(template: Template): void {
    this.change(() => this.insertTemplate(template));
  }

  // ---------- moving ----------

  moveRight(): void {
    this.change(() => {
      const { row, offset } = this.cursor;
      if (offset < row.children.length) {
        const rows = childRows(row.children[offset]);
        this.cursor = rows.length ? { row: rows[0], offset: 0 } : { row, offset: offset + 1 };
        return;
      }
      this.leave(row, +1);
    });
  }

  moveLeft(): void {
    this.change(() => {
      const { row, offset } = this.cursor;
      if (offset > 0) {
        const rows = childRows(row.children[offset - 1]);
        const last = rows[rows.length - 1];
        this.cursor = last ? { row: last, offset: last.children.length } : { row, offset: offset - 1 };
        return;
      }
      this.leave(row, -1);
    });
  }

  /** Up: from a denominator to its numerator, from an index to the exponent, from a lower limit to the upper. */
  moveUp(): void {
    this.change(() => this.vertical(-1));
  }

  moveDown(): void {
    this.change(() => this.vertical(+1));
  }

  home(): void {
    this.change(() => (this.cursor = { row: this.cursor.row, offset: 0 }));
  }

  end(): void {
    this.change(() => (this.cursor = { row: this.cursor.row, offset: this.cursor.row.children.length }));
  }

  /** Puts the cursor somewhere, as a click does. */
  setCursor(cursor: Cursor): void {
    this.change(() => (this.cursor = { row: cursor.row, offset: Math.max(0, Math.min(cursor.offset, cursor.row.children.length)) }));
  }

  // ---------- deleting ----------

  /** Backspace: deletes the atom before the cursor; a structure with something inside is entered
   * instead, from its end; at the start of an empty place, the structure goes if all of it is empty. */
  backspace(): void {
    this.change(() => {
      const { row, offset } = this.cursor;
      if (offset > 0) {
        const atom = row.children[offset - 1];
        if (atom.kind === "command" && atom.name.length > 0) {
          atom.name = atom.name.slice(0, -1);
          return;
        }
        const rows = childRows(atom);
        if (rows.some((r) => r.children.length > 0)) {
          const last = [...rows].reverse().find((r) => r.children.length > 0)!;
          this.cursor = { row: last, offset: last.children.length };
          return;
        }
        this.removeAt(row, offset - 1);
        return;
      }
      const owner = row.parent;
      if (!owner) return;
      const outer = owner.parent!;
      const at = outer.children.indexOf(owner);
      // an exponent or an index alone can go; a root's index too
      if (row.children.length === 0 && owner.kind === "scripts") {
        if (owner.sup === row) owner.sup = null;
        if (owner.sub === row) owner.sub = null;
        if (!owner.sup && !owner.sub) outer.children.splice(at, 1);
        this.cursor = { row: outer, offset: at };
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
      this.cursor = { row: outer, offset: at };
    });
  }

  /** Empties the formula. */
  clear(): void {
    this.change(() => {
      this.root.children = [];
      this.cursor = { row: this.root, offset: 0 };
    });
  }

  // ---------- undo ----------

  undo(): void {
    const state = this.undoStack.pop();
    if (state === undefined) return;
    this.redoStack.push(this.snapshot());
    this.restore(state);
    this.emit();
  }

  redo(): void {
    const state = this.redoStack.pop();
    if (state === undefined) return;
    this.undoStack.push(this.snapshot());
    this.restore(state);
    this.emit();
  }

  // ---------- internals ----------

  private id(): string {
    return `n${this.next++}`;
  }

  private row(parent: Atom | null): Row {
    return { kind: "row", id: this.id(), children: [], parent };
  }

  private change(f: () => void): void {
    const before = this.snapshot();
    f();
    if (this.snapshot() !== before) {
      const last = this.undoStack[this.undoStack.length - 1];
      if (last !== before) this.undoStack.push(before);
      if (this.undoStack.length > 200) this.undoStack.shift();
      this.redoStack = [];
    }
    this.emit();
  }

  private emit(): void {
    for (const l of this.listeners) l();
  }

  private typingCommand(): Command | null {
    const { row, offset } = this.cursor;
    const before = row.children[offset - 1];
    return before && before.kind === "command" ? before : null;
  }

  private finishCommand(command: Command): void {
    const row = command.parent!;
    const at = row.children.indexOf(command);
    row.children.splice(at, 1);
    this.cursor = { row, offset: at };
    const name = command.name;
    if (!name) return;
    const template = TEMPLATES[name];
    if (template) return this.insertTemplate(template);
    this.insertAtom({ kind: "symbol", id: this.id(), parent: null, latex: `\\${name}`, cls: SYMBOLS[name] ?? "ord" });
  }

  private insertAtom(atom: Atom): void {
    const { row, offset } = this.cursor;
    atom.parent = row;
    row.children.splice(offset, 0, atom);
    this.cursor = { row, offset: offset + 1 };
  }

  private insertTemplate(template: Template): void {
    const base = { id: this.id(), parent: null };
    let atom: Atom;
    switch (template) {
      case "frac": {
        const a: Fraction = { ...base, kind: "frac", num: null!, den: null! };
        a.num = this.row(a);
        a.den = this.row(a);
        atom = a;
        break;
      }
      case "sqrt":
      case "nroot": {
        const a: Root = { ...base, kind: "sqrt", index: null, body: null! };
        a.body = this.row(a);
        if (template === "nroot") a.index = this.row(a);
        atom = a;
        break;
      }
      case "sup":
      case "sub": {
        const { row, offset } = this.cursor;
        const before = row.children[offset - 1];
        // a second ^ on the same atom goes into the exponent it already has
        if (before && before.kind === "scripts") {
          const existing = before[template];
          if (existing) {
            this.cursor = { row: existing, offset: existing.children.length };
            return;
          }
          const r = this.row(before);
          before[template] = r;
          this.cursor = { row: r, offset: 0 };
          return;
        }
        const a: Scripts = { ...base, kind: "scripts", sup: null, sub: null };
        a[template] = this.row(a);
        atom = a;
        break;
      }
      case "paren":
      case "brackets":
      case "abs": {
        const [left, right] = template === "paren" ? ["(", ")"] : template === "brackets" ? ["[", "]"] : ["|", "|"];
        const a: Delimited = { ...base, kind: "delim", left, right, body: null! };
        a.body = this.row(a);
        atom = a;
        break;
      }
      case "sum":
      case "prod":
      case "int":
      case "lim": {
        const a: LargeOperator = { ...base, kind: "bigop", op: `\\${template}`, sub: null, sup: null };
        a.sub = this.row(a);
        if (template !== "lim") a.sup = this.row(a);
        atom = a;
        break;
      }
    }
    this.insertAtom(atom);
    const rows = childRows(atom);
    const first = rows.find((r) => r.children.length === 0) ?? rows[0];
    if (first) this.cursor = { row: first, offset: 0 };
  }

  /** `/`: the operand before the cursor (back to an operator, a relation or an opening) becomes the numerator. */
  private fractionOfOperand(): void {
    const { row, offset } = this.cursor;
    let start = offset;
    let depth = 0;
    while (start > 0) {
      const a = row.children[start - 1];
      if (a.kind === "symbol" && a.cls === "close") depth++;
      else if (a.kind === "symbol" && a.cls === "open") {
        if (depth === 0) break;
        depth--;
      } else if (depth === 0 && (a.kind === "command" || (a.kind === "symbol" && a.cls !== "ord"))) break;
      start--;
    }
    const taken = row.children.splice(start, offset - start);
    this.cursor = { row, offset: start };
    this.insertTemplate("frac");
    const frac = row.children[start] as Fraction;
    for (const a of taken) a.parent = frac.num;
    frac.num.children = taken;
    this.cursor = { row: taken.length ? frac.den : frac.num, offset: 0 };
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
    // the nearest structure around the cursor with a row on that side
    for (let row: Row = this.cursor.row; row.parent; row = row.parent.parent!) {
      const owner = row.parent;
      const order = verticalRows(owner);
      const i = order.indexOf(row);
      const target = order[i + side];
      if (i >= 0 && target) {
        this.cursor = { row: target, offset: side > 0 ? 0 : target.children.length };
        if (target.children.length) this.cursor.offset = Math.min(this.cursor.offset, target.children.length);
        return;
      }
    }
  }

  private removeAt(row: Row, i: number): void {
    row.children.splice(i, 1);
    this.cursor = { row, offset: i };
  }

  private rowLatex(row: Row, marked: boolean): string {
    if (row.children.length === 0) return marked ? `\\htmlData{m=${row.id},empty=1}{\\square}` : "";
    let out = "";
    for (const atom of row.children) out = join(out, this.atomLatex(atom, marked));
    return marked ? `\\htmlData{m=${row.id}}{${out}}` : out;
  }

  private atomLatex(atom: Atom, marked: boolean): string {
    const mark = (s: string) => (marked ? `\\htmlData{m=${atom.id}}{${s}}` : s);
    const r = (row: Row) => this.rowLatex(row, marked);
    switch (atom.kind) {
      case "symbol": {
        if (!marked) return atom.latex;
        const wrapper = CLASS_WRAPPER[atom.cls];
        const m = mark(atom.latex);
        return wrapper ? `${wrapper}{${m}}` : m;
      }
      case "command":
        return marked ? mark(`\\mathtt{\\backslash ${atom.name}}`) : "";
      case "frac":
        return mark(`\\frac{${r(atom.num)}}{${r(atom.den)}}`);
      case "sqrt":
        return mark(atom.index ? `\\sqrt[${r(atom.index)}]{${r(atom.body)}}` : `\\sqrt{${r(atom.body)}}`);
      case "delim":
        return mark(`\\left${delimiter(atom.left)}${r(atom.body)}\\right${delimiter(atom.right)}`);
      case "bigop":
        return mark(`${atom.op}${atom.sub ? `_{${r(atom.sub)}}` : ""}${atom.sup ? `^{${r(atom.sup)}}` : ""}`);
      case "scripts": {
        // on the atom before it; at the start of a row, or after another script, on an empty base
        const row = atom.parent!;
        const before = row.children[row.children.indexOf(atom) - 1];
        const base = !before || before.kind === "scripts" ? "{}" : "";
        return `${base}${atom.sup ? `^{${r(atom.sup)}}` : ""}${atom.sub ? `_{${r(atom.sub)}}` : ""}`;
      }
    }
  }

  private snapshot(): string {
    return JSON.stringify({ root: plain(this.root), cursor: this.path(this.cursor.row), offset: this.cursor.offset, next: this.next });
  }

  private restore(state: string): void {
    const s = JSON.parse(state);
    this.next = s.next;
    this.root = revive(s.root, null) as Row;
    let row = this.root;
    for (const [i, j] of s.cursor as [number, number][]) row = childRows(row.children[i])[j];
    this.cursor = { row, offset: s.offset };
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
    case "delim":
      return [atom.body];
    case "bigop":
      return [atom.sub, atom.sup].filter((r): r is Row => r !== null);
    default:
      return [];
  }
}

/** An atom's rows from top to bottom, for up and down. */
function verticalRows(atom: Atom): Row[] {
  switch (atom.kind) {
    case "frac":
      return [atom.num, atom.den];
    case "scripts":
    case "bigop":
      return [atom.sup, atom.sub].filter((r): r is Row => r !== null);
    default:
      return [];
  }
}

/** Two pieces of LaTeX one after the other, with a space only where a control word would swallow a letter. */
function join(a: string, b: string): string {
  if (!a) return b;
  return /\\[A-Za-z]+$/.test(a) && /^[A-Za-z]/.test(b) ? `${a} ${b}` : a + b;
}

function delimiter(d: string): string {
  return d === "{" ? "\\{" : d === "}" ? "\\}" : d;
}

/** The tree without parent links, for a snapshot. */
function plain(node: Row | Atom): unknown {
  const out: Record<string, unknown> = {};
  for (const [k, v] of Object.entries(node)) {
    if (k === "parent") continue;
    if (Array.isArray(v)) out[k] = v.map((c) => plain(c));
    else if (v && typeof v === "object") out[k] = plain(v as Row);
    else out[k] = v;
  }
  return out;
}

function revive(data: Record<string, unknown>, parent: Row | Atom | null): Row | Atom {
  const node = { ...data, parent } as Record<string, unknown>;
  for (const [k, v] of Object.entries(data)) {
    if (Array.isArray(v)) node[k] = v.map((c) => revive(c as Record<string, unknown>, node as unknown as Row));
    else if (v && typeof v === "object") node[k] = revive(v as Record<string, unknown>, node as unknown as Atom);
  }
  return node as unknown as Row | Atom;
}
