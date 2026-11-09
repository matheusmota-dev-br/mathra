// Reads LaTeX into the editor's structures. What it does not know stays as written, in a raw atom
// that is drawn as is and can be moved and deleted whole. In a snippet (a palette's insert), `#?` is
// an empty place and `#0` where the selection goes.

import {
  ACCENTS,
  CHAR_CLASS,
  FRACTIONS,
  LARGE_OPERATORS,
  MATRICES,
  SPACES,
  STYLES,
  TEXTS,
  classOfCommand,
  type AtomClass,
} from "./symbols.ts";

/** An atom as data: no id, no parent. What the reader makes and the clipboard and undo keep. */
export type AtomSpec =
  | { kind: "symbol"; latex: string; cls: AtomClass }
  | { kind: "char"; ch: string }
  | { kind: "text"; cmd: string; body: AtomSpec[] }
  | { kind: "frac"; cmd: string; num: AtomSpec[]; den: AtomSpec[] }
  | { kind: "sqrt"; index: AtomSpec[] | null; body: AtomSpec[] }
  | { kind: "scripts"; sup: AtomSpec[] | null; sub: AtomSpec[] | null }
  /** `auto`: plain delimiters while what they hold is small, `\left…\right` when it is tall (typed ones); else always `\left…\right` (read ones). */
  | { kind: "delim"; left: string; right: string; body: AtomSpec[]; auto?: boolean }
  | { kind: "bigop"; op: string; sub: AtomSpec[] | null; sup: AtomSpec[] | null }
  | { kind: "accent"; cmd: string; body: AtomSpec[] }
  | { kind: "style"; cmd: string; body: AtomSpec[] }
  | { kind: "matrix"; env: string; cells: AtomSpec[][][] }
  | { kind: "raw"; latex: string }
  | { kind: "command"; name: string }
  /** `#0` in a snippet: where the selection goes. */
  | { kind: "selection" };

class Reader {
  private pos = 0;
  private readonly src: string;
  private readonly auto: boolean;
  constructor(src: string, auto = false) {
    this.src = src;
    this.auto = auto;
  }

  private peek(n = 0): string {
    return this.src[this.pos + n] ?? "";
  }

  private atEnd(): boolean {
    return this.pos >= this.src.length;
  }

  private skipSpaces(): void {
    for (;;) {
      const c = this.peek();
      if (c === " " || c === "\n" || c === "\t" || c === "\r") this.pos++;
      else if (c === "%") while (!this.atEnd() && this.peek() !== "\n") this.pos++;
      else return;
    }
  }

  /** `\name` (letters) or `\x` (one other character), after the backslash. */
  private commandName(): string {
    const start = this.pos;
    if (/[A-Za-z]/.test(this.peek())) {
      while (/[A-Za-z]/.test(this.peek())) this.pos++;
      // a star belongs to the name: \operatorname*, \text* do not exist, but \cfrac* is harmless
      return this.src.slice(start, this.pos);
    }
    this.pos++;
    return this.src.slice(start, this.pos);
  }

  /** The text up to the brace that closes the one just read, kept as written. */
  private rawGroup(open = "{", close = "}"): string {
    let depth = 1;
    const start = this.pos;
    while (!this.atEnd()) {
      const c = this.peek();
      if (c === "\\") {
        this.pos += 2;
        continue;
      }
      if (c === open) depth++;
      if (c === close && --depth === 0) {
        const s = this.src.slice(start, this.pos);
        this.pos++;
        return s;
      }
      this.pos++;
    }
    return this.src.slice(start);
  }

  /** Skips a stray closing brace; false if there is none. */
  skipBrace(): boolean {
    if (this.peek() !== "}") return false;
    this.pos++;
    return true;
  }

  /** A list of atoms up to the end, a closing brace, or one of `stops` (`&`, `\\`, `\right`, `\end`). */
  list(stops: string[] = []): AtomSpec[] {
    const out: AtomSpec[] = [];
    for (;;) {
      this.skipSpaces();
      if (this.atEnd() || this.peek() === "}") return out;
      if (stops.includes("&") && this.peek() === "&") return out;
      if (this.peek() === "\\") {
        const save = this.pos;
        this.pos++;
        const name = this.commandName();
        this.pos = save;
        if ((name === "\\" && stops.includes("\\\\")) || (name === "right" && stops.includes("\\right")) || (name === "end" && stops.includes("\\end"))) return out;
      }
      this.step(out);
    }
  }

  /** One argument: a group, or a single token. */
  private argument(): AtomSpec[] {
    this.skipSpaces();
    if (this.peek() === "{") {
      this.pos++;
      const l = this.list();
      if (this.peek() === "}") this.pos++;
      return l;
    }
    if (this.atEnd()) return [];
    const out: AtomSpec[] = [];
    this.atom(out);
    return out;
  }

  /** Reads one thing and the scripts after it into `out`. */
  private step(out: AtomSpec[]): void {
    const c = this.peek();
    if (c === "^" || c === "_") {
      this.pos++;
      this.script(out, c === "^" ? "sup" : "sub");
      return;
    }
    if (c === "{") {
      // a group adds its atoms in place
      this.pos++;
      out.push(...this.list());
      if (this.peek() === "}") this.pos++;
      return;
    }
    this.atom(out);
  }

  private script(out: AtomSpec[], which: "sup" | "sub"): void {
    const arg = this.argument();
    const last = out[out.length - 1];
    if (last && last.kind === "bigop" && last[which] === null) {
      last[which] = arg;
      return;
    }
    if (last && last.kind === "scripts" && last[which] === null) {
      last[which] = arg;
      return;
    }
    out.push({ kind: "scripts", sup: which === "sup" ? arg : null, sub: which === "sub" ? arg : null });
  }

  /** One atom (no scripts) into `out`. */
  private atom(out: AtomSpec[]): void {
    const c = this.peek();
    if (c === "#" && (this.peek(1) === "?" || this.peek(1) === "0")) {
      this.pos += 2;
      if (this.peek(-1) === "0") out.push({ kind: "selection" });
      return;
    }
    if (c === "\\") {
      this.pos++;
      this.command(this.commandName(), out);
      return;
    }
    this.pos++;
    if (c === "~") return void out.push({ kind: "symbol", latex: "~", cls: "ord" });
    if (c === "&" || c === "$") return;
    if (c === "'") return void out.push({ kind: "symbol", latex: "'", cls: "ord" });
    out.push({ kind: "symbol", latex: c, cls: CHAR_CLASS[c] ?? "ord" });
  }

  private command(name: string, out: AtomSpec[]): void {
    if (FRACTIONS.has(name)) {
      const num = this.argument();
      const den = this.argument();
      out.push({ kind: "frac", cmd: `\\${name}`, num, den });
      return;
    }
    if (name === "sqrt") {
      this.skipSpaces();
      let index: AtomSpec[] | null = null;
      if (this.peek() === "[") {
        this.pos++;
        index = new Reader(this.rawGroup("[", "]"), this.auto).list();
      }
      out.push({ kind: "sqrt", index, body: this.argument() });
      return;
    }
    if (LARGE_OPERATORS.has(name)) {
      out.push({ kind: "bigop", op: `\\${name}`, sub: null, sup: null });
      this.skipSpaces();
      // \limits and \nolimits: mathra's defaults are kept
      for (const w of ["\\limits", "\\nolimits"]) if (this.src.startsWith(w, this.pos) && !/[A-Za-z]/.test(this.src[this.pos + w.length] ?? "")) this.pos += w.length;
      return;
    }
    if (name === "left") {
      const left = this.delimiter();
      const body = this.list(["\\right"]);
      let right = ".";
      if (this.src.startsWith("\\right", this.pos)) {
        this.pos += 6;
        right = this.delimiter();
      }
      out.push(this.auto ? { kind: "delim", left, right, body, auto: true } : { kind: "delim", left, right, body });
      return;
    }
    if (name === "begin") {
      this.skipSpaces();
      const save = this.pos;
      const env = this.peek() === "{" ? (this.pos++, this.rawGroup()) : "";
      if (MATRICES.has(env)) return void out.push(this.matrix(env));
      // anything else stays as written, up to its \end
      const end = this.src.indexOf(`\\end{${env}}`, this.pos);
      const stop = end < 0 ? this.src.length : end + `\\end{${env}}`.length;
      out.push({ kind: "raw", latex: `\\begin${this.src.slice(save, stop)}` });
      this.pos = stop;
      return;
    }
    if (TEXTS.has(name)) {
      this.skipSpaces();
      if (this.peek() !== "{") return void out.push({ kind: "raw", latex: `\\${name}` });
      this.pos++;
      const raw = this.rawGroup();
      const chars = readText(raw);
      out.push(chars ? { kind: "text", cmd: `\\${name}`, body: chars } : { kind: "raw", latex: `\\${name}{${raw}}` });
      return;
    }
    // an operator name is one symbol: mathra reads its argument as letters
    if (name === "operatorname") {
      const star = this.peek() === "*" ? (this.pos++, "*") : "";
      this.skipSpaces();
      const arg = this.peek() === "{" ? (this.pos++, this.rawGroup()) : "";
      // with a star, its scripts are limits, as a large operator's
      if (star) return void out.push({ kind: "bigop", op: `\\operatorname*{${arg}}`, sub: null, sup: null });
      return void out.push({ kind: "symbol", latex: `\\operatorname{${arg}}`, cls: "op" });
    }
    // \not before a relation is one symbol with it
    if (name === "not") {
      this.skipSpaces();
      let next = "";
      if (this.peek() === "\\") {
        this.pos++;
        next = `\\${this.commandName()}`;
      } else if (!this.atEnd()) next = this.src[this.pos++];
      return void out.push({ kind: "symbol", latex: `\\not${/^\\[A-Za-z]/.test(next) ? "" : " "}${next}`, cls: "rel" });
    }
    // \color{c} colors the rest of its group; \textcolor{c}{x}, its argument
    if (name === "color" || name === "textcolor") {
      this.skipSpaces();
      const color = this.peek() === "{" ? (this.pos++, this.rawGroup()) : "";
      const body = name === "color" ? this.list(["&", "\\\\", "\\right", "\\end"]) : this.argument();
      return void out.push({ kind: "style", cmd: `\\${name}{${color}}`, body });
    }
    if (ACCENTS.has(name)) return void out.push({ kind: "accent", cmd: `\\${name}`, body: this.argument() });
    if (STYLES.has(name)) return void out.push({ kind: "style", cmd: `\\${name}`, body: this.argument() });
    if (SPACES.has(name)) return void out.push({ kind: "symbol", latex: `\\${name}`, cls: "ord" });
    if (name === "\\") return void out.push({ kind: "raw", latex: "\\\\" });
    if (/^[{}|#$%&_]$/.test(name)) return void out.push({ kind: "symbol", latex: `\\${name}`, cls: classOfCommand(name) });
    // an unknown command right before an argument keeps its arguments, as written
    if (!(name in KNOWN_NO_ARGUMENT) && (this.peek() === "{" || this.peek() === "[")) {
      let latex = `\\${name}`;
      while (this.peek() === "{" || this.peek() === "[") {
        const open = this.peek();
        const close = open === "{" ? "}" : "]";
        this.pos++;
        latex += `${open}${this.rawGroup(open, close)}${close}`;
      }
      out.push({ kind: "raw", latex });
      return;
    }
    out.push({ kind: "symbol", latex: `\\${name}`, cls: classOfCommand(name) });
  }

  private delimiter(): string {
    this.skipSpaces();
    if (this.peek() === "\\") {
      this.pos++;
      return `\\${this.commandName()}`;
    }
    const c = this.peek();
    this.pos++;
    return c || ".";
  }

  private matrix(env: string): AtomSpec {
    const cells: AtomSpec[][][] = [[]];
    for (;;) {
      const cell = this.list(["&", "\\\\", "\\end"]);
      cells[cells.length - 1].push(cell);
      this.skipSpaces();
      if (this.peek() === "&") {
        this.pos++;
        continue;
      }
      if (this.src.startsWith("\\\\", this.pos)) {
        this.pos += 2;
        // \\[2pt]: the space is dropped
        if (this.peek() === "[") {
          this.pos++;
          this.rawGroup("[", "]");
        }
        cells.push([]);
        continue;
      }
      if (this.src.startsWith("\\end", this.pos)) {
        this.pos += 4;
        this.skipSpaces();
        if (this.peek() === "{") {
          this.pos++;
          this.rawGroup();
        }
      }
      break;
    }
    // a trailing \\ leaves an empty last line
    const last = cells[cells.length - 1];
    if (cells.length > 1 && last.length === 1 && last[0].length === 0) cells.pop();
    const width = Math.max(...cells.map((r) => r.length));
    for (const r of cells) while (r.length < width) r.push([]);
    return { kind: "matrix", env, cells };
  }
}

/** Commands that take no argument, even right before a brace (`\alpha{}` is α then a group). */
const KNOWN_NO_ARGUMENT: Record<string, true> = {};
for (const n of "alpha beta gamma delta epsilon varepsilon zeta eta theta vartheta iota kappa lambda mu nu xi pi varpi rho varrho sigma varsigma tau upsilon phi varphi chi psi omega Gamma Delta Theta Lambda Xi Pi Sigma Upsilon Phi Psi Omega infty partial nabla hbar ell emptyset varnothing forall exists nexists neg prime cdots ldots vdots ddots dots".split(" ")) KNOWN_NO_ARGUMENT[n] = true;

/** The characters of a `\text{…}` argument, or null if it holds math or a command the editor does not know. */
function readText(raw: string): AtomSpec[] | null {
  const out: AtomSpec[] = [];
  for (let i = 0; i < raw.length; i++) {
    const c = raw[i];
    if (c === "#" && (raw[i + 1] === "0" || raw[i + 1] === "?")) {
      if (raw[i + 1] === "0") out.push({ kind: "selection" });
      i++;
      continue;
    }
    if (c === "$") return null;
    if (c === "\\") {
      const rest = raw.slice(i + 1);
      const m = /^(textbackslash|[{}#$%&_ ]|\^\{\}|~\{\})(\{\})?/.exec(rest);
      if (!m) return null;
      const name = m[1];
      out.push({ kind: "char", ch: name === "textbackslash" ? "\\" : name === "^{}" ? "^" : name === "~{}" ? "~" : name });
      i += m[0].length;
      continue;
    }
    if (c === "{" || c === "}") continue;
    out.push({ kind: "char", ch: c === "~" ? " " : c });
  }
  return out;
}

/** Reads LaTeX (or a snippet with `#?` and `#0`) into atoms. With `autoDelimiters`, `\left…\right`
 * pairs grow only when what they hold is tall (what the editor's own buttons and keys insert). */
export function readLatex(latex: string, options: { autoDelimiters?: boolean } = {}): AtomSpec[] {
  const r = new Reader(latex, options.autoDelimiters ?? false);
  const out: AtomSpec[] = [];
  // a stray closing brace is skipped, not the end of everything
  for (;;) {
    out.push(...r.list());
    if (!r.skipBrace()) return out;
  }
}
