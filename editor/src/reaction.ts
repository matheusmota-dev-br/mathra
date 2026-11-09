// Reactions (mhchem's \ce) as pieces: species (coefficient, formula, state), arrows (with what goes
// over and under them) and signs (+, ↑, ↓), read from the text of a \ce and written back.

/** A piece of a reaction, its texts as written. */
export type PieceData =
  | { kind: "species"; coef: string; formula: string; state: string }
  | { kind: "rxarrow"; arrow: string; above: string | null; below: string | null }
  | { kind: "rxtoken"; text: string };

/** mhchem's arrows, the longest first (so `<=>>` is not read as `<=>`). */
export const REACTION_ARROWS = ["<=>>", "<<=>", "<-->", "<=>", "<->", "->", "<-", "="];
const STATES = /^(.*?)\((s|l|g|aq|cr|sln|ppt|gas|liq|sol)\)$/;
const NUMBER = /^(\d+(?:[.,]\d+)?|\d+\/\d+)$/;
const SIGNS = new Set(["+", "^", "v"]);

/** Splits at spaces outside braces, brackets and `$…$`. */
function tokens(src: string): string[] {
  const out: string[] = [];
  let depth = 0;
  let math = false;
  let cur = "";
  for (let i = 0; i < src.length; i++) {
    const c = src[i];
    if (c === "\\") {
      cur += c + (src[i + 1] ?? "");
      i++;
      continue;
    }
    if (c === "$") math = !math;
    else if (!math && (c === "{" || c === "[")) depth++;
    else if (!math && (c === "}" || c === "]")) depth--;
    if (/\s/.test(c) && depth === 0 && !math) {
      if (cur) out.push(cur);
      cur = "";
      continue;
    }
    cur += c;
  }
  if (cur) out.push(cur);
  return out;
}

/** The text of a bracket group at `s[i]`, and where it ends. */
function bracket(s: string, i: number): { text: string; end: number } | null {
  if (s[i] !== "[") return null;
  let depth = 0;
  for (let j = i; j < s.length; j++) {
    if (s[j] === "\\") {
      j++;
      continue;
    }
    if (s[j] === "[" || s[j] === "{") depth++;
    else if (s[j] === "]" || s[j] === "}") {
      depth--;
      if (depth === 0 && s[j] === "]") return { text: s.slice(i + 1, j), end: j + 1 };
    }
  }
  return null;
}

/** Reads the text of a \ce into pieces; null when the editor cannot take it apart (it stays a block). */
export function readReaction(src: string): PieceData[] | null {
  if (/[&]|\\\\/.test(src)) return null;
  const out: PieceData[] = [];
  const ts = tokens(src);
  for (let k = 0; k < ts.length; k++) {
    const t = ts[k];
    const arrow = REACTION_ARROWS.find((a) => t.startsWith(a));
    if (arrow && (t.length === arrow.length || t[arrow.length] === "[")) {
      let rest = t.slice(arrow.length);
      const a = bracket(rest, 0);
      let above: string | null = null;
      let below: string | null = null;
      if (a) {
        above = a.text;
        rest = rest.slice(a.end);
        const b = bracket(rest, 0);
        if (b) {
          below = b.text;
          rest = rest.slice(b.end);
        }
      }
      if (rest) return null;
      out.push({ kind: "rxarrow", arrow, above, below });
      continue;
    }
    if (SIGNS.has(t)) {
      out.push({ kind: "rxtoken", text: t });
      continue;
    }
    // a coefficient written apart: 5 O2
    if (NUMBER.test(t) && ts[k + 1] && !SIGNS.has(ts[k + 1]) && !REACTION_ARROWS.some((x) => ts[k + 1].startsWith(x))) {
      out.push(species(t, ts[k + 1]));
      k++;
      continue;
    }
    // a coefficient written on: 2H2O
    const m = /^(\d+)([A-Z(\[].*)$/.exec(t);
    if (m) {
      out.push(species(m[1], m[2]));
      continue;
    }
    out.push(species("", t));
  }
  return out;
}

function species(coef: string, rest: string): PieceData {
  const m = STATES.exec(rest);
  return m && m[1] ? { kind: "species", coef, formula: m[1], state: m[2] } : { kind: "species", coef, formula: rest, state: "" };
}

/** Writes pieces back as the text of a \ce (a coefficient apart from its formula, as mhchem draws it the same). */
export function writeReaction<T>(pieces: T[], piece: (p: T) => string): string {
  return pieces.map(piece).filter(Boolean).join(" ");
}

/** The text of a species: coefficient, formula and state. */
export function speciesText(coef: string, formula: string, state: string): string {
  return `${coef ? `${coef} ` : ""}${formula}${state ? `(${state})` : ""}`;
}

/** The text of an arrow with what goes over and under it. */
export function arrowText(arrow: string, above: string, below: string): string {
  if (!above && !below) return arrow;
  return `${arrow}[${above}]${below ? `[${below}]` : ""}`;
}
