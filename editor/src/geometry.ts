// From the boxes mathra returns for the marked LaTeX to where the cursor is drawn and where a click
// lands. Everything in the SVG's own coordinates (its viewBox, y down).

import { childRows, type Atom, type Cursor, type Editor, type Row } from "./model.ts";

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

/** Where an atom is drawn. A script has no box of its own: it is where its rows are. */
export function atomRect(atom: Atom, boxes: Map<string, Box>): Rect | null {
  if (atom.kind === "scripts") return union(childRows(atom).map((r) => boxes.get(r.id)).filter((b): b is Box => !!b));
  return boxes.get(atom.id) ?? null;
}

export function rowRect(row: Row, boxes: Map<string, Box>): Rect | null {
  return boxes.get(row.id) ?? null;
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
  const x = after ? atomRect(after, boxes)?.x : (() => {
    const r = atomRect(before!, boxes);
    return r ? r.x + r.width : undefined;
  })();
  if (x === undefined) return null;
  // the height of a neighbour symbol, so the cursor beside a tall fraction stays the size of a letter
  const neighbour = [before, after].find((a) => a && a.kind === "symbol") ?? before ?? after;
  const nb = neighbour ? atomRect(neighbour, boxes) : null;
  const v = nb && neighbour!.kind === "symbol" ? nb : rb;
  return { x: x - thickness / 2, y: v.y, width: thickness, height: v.height };
}

function contains(r: Rect, x: number, y: number): boolean {
  return x >= r.x && x <= r.x + r.width && y >= r.y && y <= r.y + r.height;
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
