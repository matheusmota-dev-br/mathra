// Diagrams edited as boxes: cells, arrows made from the keyboard, their styles and labels.

import { test } from "node:test";
import assert from "node:assert/strict";
import { Editor, type Diagram } from "../src/model.ts";
import { readDiagram, writeDiagram } from "../src/diagram.ts";

const diagramOf = (e: Editor) => e.root.children.find((a) => a.kind === "diagram") as Diagram;

test("\\tikzcd typed: one empty cell, the cursor in it", () => {
  const e = new Editor();
  e.typeText("\\tikzcd ");
  const d = diagramOf(e);
  assert.ok(d);
  assert.equal(e.cursor.row, d.cells[0][0]);
});

test("Alt+arrow from a cell: an arrow to the neighbour, the grid grows at the edge, the cursor in the label", () => {
  const e = new Editor();
  e.typeText("\\tikzcd A");
  e.arrowFromCursor("right");
  e.typeText("f");
  const d = diagramOf(e);
  assert.equal(d.cells[0].length, 2);
  assert.equal(e.toLatex(), '\\begin{tikzcd}A \\arrow[r, "f"] & {}\\end{tikzcd}');
  // into the new cell: right from the label goes to the arrow's target
  e.moveRight();
  e.typeText("B");
  e.arrowFromCursor("down", "dashed");
  e.typeText("g");
  assert.equal(d.cells.length, 2);
  assert.equal(e.toLatex(), '\\begin{tikzcd}A \\arrow[r, "f"] & B \\arrow[d, "g", dashed] \\\\ & {}\\end{tikzcd}');
  // up from the top line adds a line above: the arrows follow their cells
  e.moveLeft();
  e.moveLeft();
  e.arrowFromCursor("up");
  assert.equal(d.cells.length, 3);
  assert.deepEqual(d.arrows.map((a) => [a.from, a.to]), [[[1, 0], [1, 1]], [[1, 1], [2, 1]], [[1, 1], [0, 1]]]);
});

test("an arrow's style, bend and label side; Backspace in its empty label takes it away", () => {
  const e = new Editor('\\begin{tikzcd} A \\arrow[r, "f"] & B \\end{tikzcd}');
  const d = diagramOf(e);
  e.selectArrow(d, 0);
  assert.equal(e.arrowAtCursor()?.index, 0);
  e.setArrowStyle("hook");
  e.setArrowBend("left");
  e.swapArrowLabel();
  assert.equal(e.toLatex(), '\\begin{tikzcd}A \\arrow[r, "f"\', hook, bend left] & B\\end{tikzcd}');
  e.backspace();
  e.backspace();
  assert.equal(d.arrows.length, 0);
  assert.equal(e.toLatex(), "\\begin{tikzcd}A & B\\end{tikzcd}");
});

test("amscd stays amscd while it can; a diagonal arrow makes it tikz-cd", () => {
  const e = new Editor("\\begin{CD} A @>f>> B \\\\ @VgVV @VVhV \\\\ C @>k>> D \\end{CD}");
  assert.equal(e.toLatex(), "\\begin{CD} A @>f>> B \\\\ @VgVV @VVhV \\\\ C @>k>> D \\end{CD}");
  const d = diagramOf(e);
  e.addArrow(d, [0, 0], [1, 1]);
  assert.match(e.toLatex(), /^\\begin\{tikzcd\}/);
});

test("lines and columns added or removed: the arrows follow, or go with them", () => {
  const e = new Editor("\\begin{tikzcd} A \\arrow[r] & B \\arrow[d] \\\\ & C \\end{tikzcd}");
  const d = diagramOf(e);
  e.setCursor({ row: d.cells[0][0], offset: 1 });
  e.addColumn();
  assert.deepEqual(d.arrows.map((a) => [a.from, a.to]), [[[0, 0], [0, 2]], [[0, 2], [1, 2]]]);
  e.setCursor({ row: d.cells[1][2], offset: 1 });
  e.removeRow();
  assert.deepEqual(d.arrows.map((a) => [a.from, a.to]), [[[0, 0], [0, 2]]]);
});

test("xymatrix and tikz-cd read and written back the same", () => {
  for (const latex of ["\\xymatrix{A \\ar@{^{(}->}[r]^{\\iota} \\ar[d]_{\\alpha} & B \\ar@{->>}[d] \\\\ C \\ar@{=}[r] & D}", '\\begin{tikzcd}T \\arrow[dr, "\\exists!" description, dashed] & \\\\ & P\\end{tikzcd}']) {
    const d = readDiagram(latex)!;
    assert.equal(writeDiagram(d, (c) => c, (l) => l).replace(/\s+/g, ""), latex.replace(/\s+/g, ""));
  }
});

test("moving: right from a cell to the next cell, down to the cell below, out at the end", () => {
  const e = new Editor("\\begin{tikzcd} A & B \\\\ C & D \\end{tikzcd}x");
  const d = diagramOf(e);
  e.setCursor({ row: d.cells[0][0], offset: 1 });
  e.moveRight();
  assert.equal(e.cursor.row, d.cells[0][1]);
  e.moveDown();
  assert.equal(e.cursor.row, d.cells[1][1]);
  e.end();
  e.moveRight();
  assert.equal(e.cursor.row, e.root);
});

test("Alt+left from the first column adds a column before it", () => {
  const e = new Editor("\\begin{tikzcd} A \\end{tikzcd}");
  const d = diagramOf(e);
  e.setCursor({ row: d.cells[0][0], offset: 1 });
  e.arrowFromCursor("left");
  assert.equal(e.toLatex(), "\\begin{tikzcd}{} & A \\arrow[l]\\end{tikzcd}");
});

test("Delete at the end of a label leaves it; Backspace in a new empty diagram takes it away", () => {
  const e = new Editor('\\begin{tikzcd} A \\arrow[r, "f"] & B \\end{tikzcd}');
  const d = diagramOf(e);
  e.selectArrow(d, 0);
  assert.deepEqual([e.cursor.row, e.cursor.offset], [d.arrows[0].label, 1]);
  e.deleteForward();
  assert.equal(e.cursor.row, d.cells[0][1]);
  assert.equal(e.toLatex(), '\\begin{tikzcd}A \\arrow[r, "f"] & B\\end{tikzcd}');
  const f = new Editor();
  f.typeText("\\tikzcd ");
  f.backspace();
  assert.equal(f.toLatex(), "");
  assert.deepEqual([f.cursor.row, f.cursor.offset], [f.root, 0]);
});

test("read back the same: amscd's arrows in every direction, braces and commands in its cells", () => {
  const same = (latex: string) => writeDiagram(readDiagram(latex)!, (c) => c, (l) => l);
  const back = "\\begin{CD} A @<f<g< B \\\\ @AaAbA @| \\\\ C @= D \\end{CD}";
  assert.equal(same(back), back);
  // leftward and upward: the first label on its own side of the way the arrow goes
  assert.deepEqual(readDiagram(back)!.arrows.slice(0, 2).map((a) => [a.from, a.to, a.label, a.swapLabel]), [[[0, 1], [0, 0], "g", "f"], [[1, 0], [0, 0], "b", "a"]]);
  const groups = "\\begin{CD} \\alpha @>{f_{1}}>> {B} \\\\ @. @VVV \\\\ C @. D \\end{CD}";
  assert.equal(same(groups), groups);
  // a group never closed: beyond what the editor reads
  assert.equal(readDiagram("\\begin{CD} {A @>>> B \\end{CD}"), null);
});

test("read back the same: xymatrix's bends and labels of one token", () => {
  const d = readDiagram("\\xymatrix{A \\ar@/^/[r]^\\alpha_f & B \\ar@/_/[l]}")!;
  assert.deepEqual(d.arrows.map((a) => [a.bend, a.label, a.swapLabel]), [["left", "\\alpha", "f"], ["right", null, null]]);
  assert.equal(writeDiagram(d, (c) => c, (l) => l), "\\xymatrix{A \\ar@/^/[r]^{\\alpha}_{f} & B \\ar@/_/[l]}");
});

test("tikz-cd: a second label on one side kept as written; options on the diagram or a bracket never closed, a block", () => {
  const d = readDiagram('\\begin{tikzcd} A \\arrow[r, "f", "g"] & B \\end{tikzcd}')!;
  assert.deepEqual([d.arrows[0].label, d.arrows[0].extra], ["f", ['"g"']]);
  assert.equal(writeDiagram(d, (c) => c, (l) => l), '\\begin{tikzcd}A \\arrow[r, "f", "g"] & B\\end{tikzcd}');
  assert.equal(readDiagram("\\begin{tikzcd}[sep=large] A \\end{tikzcd}"), null);
  assert.equal(readDiagram("\\begin{tikzcd} A \\arrow[r & B \\end{tikzcd}"), null);
});

test("Backspace at the start of a cell goes to the cell before", () => {
  const e = new Editor("\\begin{tikzcd} A & B \\end{tikzcd}");
  const d = diagramOf(e);
  e.setCursor({ row: d.cells[0][1], offset: 0 });
  e.backspace();
  assert.deepEqual([e.cursor.row, e.cursor.offset], [d.cells[0][0], 1]);
  assert.equal(e.toLatex(), "\\begin{tikzcd}A & B\\end{tikzcd}");
});
