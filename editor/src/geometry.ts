// From the boxes mathra returns for the marked LaTeX to where the cursor is drawn and where a click
// lands. Everything in the SVG's own coordinates (its viewBox, y down).

import { childRows, markedRun, type Atom, type Cursor, type Editor, type Row, type Selection } from "./model.ts";

/** A box as `render(…).boxes` gives it (only the fields used here). */
export interface Box {
  attributes: Record<string, string>;
  x: number;
  y: number;
  width: number;
  height: number;
  baseline: number;
}

export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

/** The boxes by the id of what they mark (`data-m`). */
export function boxesById(boxes: Box[]): Map<string, Box> {
  const map = new Map<string, Box>();
  for (const b of boxes) {
    const id = b.attributes["data-m"];
    if (id) map.set(id, b);
  }
  return map;
}

function union(rects: Rect[]): Rect | null {
  if (!rects.length) return null;
  const x0 = Math.min(...rects.map((r) => r.x));
  const y0 = Math.min(...rects.map((r) => r.y));
  const x1 = Math.max(...rects.map((r) => r.x + r.width));
  const y1 = Math.max(...rects.map((r) => r.y + r.height));
  return { x: x0, y: y0, width: x1 - x0, height: y1 - y0 };
}

/** Where an atom is drawn. A script has no box of its own (it is where its rows are), nor small
 * delimiters (each side has one, `<id>l` and `<id>r`). */
export function atomRect(atom: Atom, boxes: Map<string, Box>): Rect | null {
  const own = boxes.get(atom.id);
  if (own) return own;
  const inNumber = numberPart(atom, boxes);
  if (inNumber) return inNumber;
  const parts: Rect[] = [];
  for (const id of [`${atom.id}l`, `${atom.id}r`]) {
    const b = boxes.get(id);
    if (b) parts.push(b);
  }
  for (const r of childRows(atom)) {
    const b = rowRect(r, boxes);
    if (b) parts.push(b);
  }
  return union(parts);
}

/** A digit or separator of a number drawn as one group (`<first id>n`): its share of the group's
 * width, a separator taking half a digit (as in the math fonts, whose digits all have one width). */
function numberPart(atom: Atom, boxes: Map<string, Box>): Rect | null {
  const row = atom.parent;
  if (!row) return null;
  const atoms = row.children;
  const k = atoms.indexOf(atom);
  for (let i = k; i >= 0; i--) {
    const group = boxes.get(`${atoms[i].id}n`);
    if (!group) continue;
    const run = markedRun(atoms, i, true) ?? markedRun(atoms, i, false);
    if (!run || !run.includes(atom)) return null;
    const weight = (a: Atom) => (a.kind === "symbol" && /^[0-9]$/.test(a.latex) ? 1 : 0.5);
    const total = run.reduce((s, a) => s + weight(a), 0);
    const before = run.slice(0, run.indexOf(atom)).reduce((s, a) => s + weight(a), 0);
    const unit = group.width / total;
    return { x: group.x + before * unit, y: group.y, width: weight(atom) * unit, height: group.height };
  }
  return null;
}

/** Where a row is drawn: its own box, or where its atoms are. */
export function rowRect(row: Row, boxes: Map<string, Box>): Rect | null {
  return boxes.get(row.id) ?? union(row.children.map((a) => atomRect(a, boxes)).filter((r): r is Rect => !!r));
}

/** Where the selection is: one rectangle over the selected atoms. */
export function selectionRect(selection: Selection | null, boxes: Map<string, Box>): Rect | null {
  if (!selection) return null;
  return union(selection.row.children.slice(selection.start, selection.end).map((a) => atomRect(a, boxes)).filter((r): r is Rect => !!r));
}

/** The cursor as a thin rectangle: at the left edge of the atom after it (or the right edge of the
 * last one), as tall as the symbol beside it; in an empty place, in the middle of the placeholder. */
export function caretRect(cursor: Cursor, boxes: Map<string, Box>, thickness = 40): Rect | null {
  const { row, offset } = cursor;
  const rb = rowRect(row, boxes);
  if (!rb) return null;
  if (row.children.length === 0) return { x: rb.x + rb.width / 2 - thickness / 2, y: rb.y, width: thickness, height: rb.height };
  const after = row.children[offset];
  const before = row.children[offset - 1];
  // an atom without a box (LaTeX that acts on the alignment, kept unmarked): the neighbour's edge
  const leftEdge = after ? atomRect(after, boxes)?.x : undefined;
  const rightOfBefore = before ? (() => {
    const r = atomRect(before, boxes);
    return r ? r.x + r.width : undefined;
  })() : undefined;
  const x = leftEdge ?? rightOfBefore ?? rb.x;
  // the height of a neighbour symbol, so the cursor beside a tall fraction stays the size of a letter
  const neighbour = [before, after].find((a) => a && (a.kind === "symbol" || a.kind === "char")) ?? before ?? after;
  const nb = neighbour ? atomRect(neighbour, boxes) : null;
  const v = nb && (neighbour!.kind === "symbol" || neighbour!.kind === "char") && nb.height > 0 ? nb : rb;
  return { x: x - thickness / 2, y: v.y, width: thickness, height: v.height };
}

function contains(r: Rect, x: number, y: number): boolean {
  return x >= r.x && x <= r.x + r.width && y >= r.y && y <= r.y + r.height;
}

/** The innermost block of LaTeX (a raw atom) drawn at (x, y), if any. */
export function blockAt(editor: Editor, boxes: Map<string, Box>, x: number, y: number): Atom | null {
  let best: Atom | null = null;
  let area = Infinity;
  for (const row of editor.rows())
    for (const a of row.children) {
      if (a.kind !== "raw") continue;
      const r = atomRect(a, boxes);
      if (r && contains(r, x, y) && r.width * r.height < area) {
        best = a;
        area = r.width * r.height;
      }
    }
  return best;
}

/** Where a click at (x, y) puts the cursor: in the innermost row whose box holds the point (the
 * formula itself if none), before the first atom whose middle is right of the point. */
export function cursorAt(editor: Editor, boxes: Map<string, Box>, x: number, y: number): Cursor {
  let best: Row = editor.root;
  let bestArea = Infinity;
  for (const row of editor.rows()) {
    const r = rowRect(row, boxes);
    if (!r || !contains(r, x, y) || row === editor.root) continue;
    const area = r.width * r.height;
    if (area <= bestArea) {
      best = row;
      bestArea = area;
    }
  }
  const children = best.children;
  let offset = children.length;
  for (let i = 0; i < children.length; i++) {
    const r = atomRect(children[i], boxes);
    if (r && x < r.x + r.width / 2) {
      offset = i;
      break;
    }
  }
  return { row: best, offset };
}
