// Schemes (chemfig's \schemestart) edited as pieces: structures, arrows with conditions, +.

import { test } from "node:test";
import assert from "node:assert/strict";
import { Editor, type Reaction } from "../src/model.ts";
import { drawing } from "./drawing.ts";
import { render } from "../../npm/pkg/node/index.mjs";

const editor = (latex = "") => {
  const e = new Editor();
  e.parseChem = (l) => render(l).tree;
  if (latex) e.setLatex(latex);
  return e;
};
const schemeOf = (e: Editor) => e.root.children.find((a) => a.kind === "reaction") as Reaction;
const same = (a: string, b: string) => JSON.stringify(drawing(render(a).svg)) === JSON.stringify(drawing(render(b).svg));

test("a scheme reads into pieces; written back and marked, it draws the same", () => {
  for (const src of ["\\schemestart \\chemfig{A-B} \\arrow{->[heat][cat]} \\chemfig{C=D} \\+ \\chemfig{E} \\schemestop", "\\schemestart \\chemfig{*6(-=-=-=)} \\arrow \\chemfig{*6(-=-=(-NO_2)-=)} \\schemestop", "\\schemestart \\chemfig{A}\\arrow{<=>}\\chemfig{B}\\schemestop"]) {
    const e = editor(src);
    const s = schemeOf(e);
    assert.ok(s?.scheme, src);
    assert.ok(same(src, e.toLatex()), `written back ${e.toLatex()}`);
    assert.ok(same(src, e.toLatex({ marked: true })), `marked ${src}`);
  }
  // with what the editor cannot take apart, a block as written
  const raw = editor("\\schemestart \\chemfig{A}\\arrow(a--b){->}\\chemfig{B}\\schemestop");
  assert.equal(raw.root.children[0].kind, "raw");
});

test("\\schemestart typed: a structure picked; a space leaves it, -> an arrow, [ its conditions, a letter the next structure", () => {
  const e = editor();
  e.typeText("\\schemestart ");
  assert.ok(e.chemSelection());
  e.typeText("A");
  e.addChemBond(0);
  e.typeText("B");
  e.typeText(" ->[heat]");
  e.typeText("C");
  assert.equal(e.toLatex(), "\\schemestart \\chemfig{A-[:0]B} \\arrow{->[heat]} \\chemfig{C} \\schemestop");
  assert.equal(render(e.toLatex()).warnings.length, 0);
  assert.ok(same(e.toLatex(), e.toLatex({ marked: true })));
});

test("the palette's ring-to-ring scheme: the first R picked, conditions typed in place", () => {
  const e = editor();
  e.insertLatex("\\schemestart \\chemfig{*6(-=-=(-R)-=)} \\arrow{->[][]} \\chemfig{*6(-=-=(-R)-=)} \\schemestop");
  e.typeText("H");
  const s = schemeOf(e);
  const arrow = s.body.children.find((p) => p.kind === "rxarrow")!;
  if (arrow.kind !== "rxarrow") throw new Error();
  e.setCursor({ row: arrow.above, offset: 0 });
  e.typeText("HNO3");
  e.setCursor({ row: arrow.below, offset: 0 });
  e.typeText("H2SO4");
  assert.equal(e.toLatex(), "\\schemestart \\chemfig{*6(-=-=(-H)-=)} \\arrow{->[HNO3][H2SO4]} \\chemfig{*6(-=-=(-R)-=)} \\schemestop");
});

test("+ from the palette in a scheme: a new structure after it, its atom picked; what a scheme cannot draw is left out", () => {
  const e = editor("\\schemestart \\chemfig{A} \\schemestop");
  const s = schemeOf(e);
  e.cursor = { row: s.body, offset: 1 };
  e.addReactionSign("+");
  assert.equal(e.toLatex(), "\\schemestart \\chemfig{A} \\+ \\chemfig{} \\schemestop");
  assert.equal(e.chemSelection()?.index, 0);
  e.typeText("B");
  assert.equal(e.toLatex(), "\\schemestart \\chemfig{A} \\+ \\chemfig{B} \\schemestop");
  // mhchem's pieces and signs other than + have no place in chemfig's scheme
  e.cursor = { row: s.body, offset: 3 };
  e.insertLatex("\\ce{C}");
  e.addReactionSign("^");
  assert.deepEqual(s.body.children.map((p) => p.kind), ["raw", "rxtoken", "raw", "species", "rxtoken"]);
  assert.equal(e.toLatex(), "\\schemestart \\chemfig{A} \\+ \\chemfig{B} \\schemestop");
});
