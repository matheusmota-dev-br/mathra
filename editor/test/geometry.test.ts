// Where things are drawn when they carry no mark of their own: empty cells of a diagram, arrows of a scheme.

import { test } from "node:test";
import assert from "node:assert/strict";
import { Editor, type Diagram, type Reaction } from "../src/model.ts";
import { atomRect, boxesById, cellRect, rowRect, type Box } from "../src/geometry.ts";
import { render } from "../../npm/pkg/node/index.mjs";

const drawn = (e: Editor) => {
  // the cursor outside, so empty cells are not marked
  e.cursor = { row: e.root, offset: e.root.children.length };
  return boxesById(render(e.toLatex({ marked: true })).boxes as Box[]);
};
const centre = (r: { x: number; width: number }) => r.x + r.width / 2;

test("an empty column between two others: in step with them", () => {
  const e = new Editor("\\begin{tikzcd} A & & C \\\\ D & & F \\end{tikzcd}");
  const d = e.root.children[0] as Diagram;
  const boxes = drawn(e);
  const a = rowRect(d.cells[0][0], boxes)!;
  const c = rowRect(d.cells[0][2], boxes)!;
  const mid = cellRect(d, 0, 1, boxes)!;
  assert.equal(centre(mid), (centre(a) + centre(c)) / 2);
  // its line's height: the middle of the cells known in it
  assert.equal(mid.y + mid.height / 2, (a.y + a.height / 2 + c.y + c.height / 2) / 2);
});

test("an empty column beside one known: tikz-cd's spacing from it", () => {
  const e = new Editor("\\begin{tikzcd} A & \\\\ C & \\end{tikzcd}");
  const d = e.root.children[0] as Diagram;
  const boxes = drawn(e);
  const c = rowRect(d.cells[1][0], boxes)!;
  const r = cellRect(d, 1, 1, boxes)!;
  assert.equal(centre(r), centre(c) + 2400);
  assert.equal(r.y + r.height / 2, c.y + c.height / 2);
});

test("a scheme's arrow with nothing drawn before it: from its labels, or an arrow's usual length", () => {
  const e = new Editor("\\schemestart \\+ \\arrow{->[heat]} \\chemfig{B} \\schemestop");
  const s = e.root.children[0] as Reaction;
  const [, arrow, b] = s.body.children;
  if (arrow.kind !== "rxarrow") throw new Error("an arrow");
  const box = (id: string, x: number, width: number): [string, Box] => [id, { attributes: { "data-m": id }, x, y: -600, width, height: 700, baseline: 0 }];
  const structure = box(`${b.id}a0`, 2000, 600);
  // its label drawn: from the label's left edge to the structure
  const labelled = atomRect(arrow, new Map([structure, box(arrow.above.id, 500, 1000)]));
  assert.deepEqual(labelled, { x: 500, y: -600, width: 1500, height: 700 });
  // nothing but the structure: the usual length before it
  const bare = atomRect(arrow, new Map([structure]));
  assert.deepEqual(bare, { x: 400, y: -600, width: 1600, height: 700 });
});
