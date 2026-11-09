// Reactions (\ce) edited as pieces: species (coefficient, formula, state), arrows with conditions, signs.

import { test } from "node:test";
import assert from "node:assert/strict";
import { Editor, type Reaction, type Species } from "../src/model.ts";
import { drawing } from "./drawing.ts";
import { render } from "../../npm/pkg/node/index.mjs";

const reactionOf = (e: Editor) => e.root.children.find((a) => a.kind === "reaction") as Reaction;
const same = (a: string, b: string) => JSON.stringify(drawing(render(a).svg)) === JSON.stringify(drawing(render(b).svg));

test("typing a reaction builds species, signs and an arrow", () => {
  const e = new Editor();
  e.typeText("\\ce 2H2 + O2 -> 2H2O");
  const r = reactionOf(e);
  assert.ok(r);
  assert.deepEqual(r.body.children.map((p) => p.kind), ["species", "rxtoken", "species", "rxarrow", "species"]);
  assert.equal(e.toLatex(), "\\ce{2 H2 + O2 -> 2 H2O}");
});

test("states, an arrow with conditions over and under", () => {
  const e = new Editor();
  e.typeText("\\ce CaCO3(s) ->[900 C][CO2] CaO(s)");
  assert.equal(e.toLatex(), "\\ce{CaCO3(s) ->[900 C][CO2] CaO(s)}");
});

test("a \\ce read back becomes pieces; marked draws as unmarked", () => {
  for (const src of ["\\ce{C3H8(g) + 5 O2(g) -> 3 CO2(g) + 4 H2O(l)}", "\\ce{2 NO2(g) <=> N2O4(g)}", "\\ce{CaCO3(s) ->[\\Delta] CaO(s) + CO2 ^}", "\\ce{Fe^3+ + e- -> Fe^2+}"]) {
    const e = new Editor();
    e.setLatex(src);
    assert.ok(reactionOf(e), src);
    assert.equal(e.toLatex(), src);
    e.cursor = { row: e.root, offset: e.root.children.length };
    assert.ok(same(e.toLatex(), e.toLatex({ marked: true })), `marks moved ${src}`);
    const boxes = render(e.toLatex({ marked: true })).boxes;
    assert.ok(boxes.length >= reactionOf(e).body.children.length, src);
  }
});

test("undo through a reaction", () => {
  const e = new Editor();
  e.typeText("\\ce A + B");
  e.undo();
  assert.notEqual(e.toLatex(), "\\ce{A + B}");
});

test("palette: a new reaction, then pieces into it (an arrow with conditions puts the cursor over it)", () => {
  const e = new Editor();
  e.insertLatex("\\ce{#?}");
  const r = reactionOf(e);
  assert.equal(e.cursor.row, r.body);
  e.typeText("CH4");
  e.insertLatex("\\ce{+}");
  e.typeText("2O2");
  e.insertLatex("\\ce{->[][]}");
  e.typeText("Δ");
  e.moveRight();
  e.moveRight();
  assert.equal(r.body.children.length, 4);
  e.cursor = { row: r.body, offset: r.body.children.length };
  e.typeText("CO2");
  assert.equal(e.toLatex(), "\\ce{CH4 + 2 O2 ->[Δ] CO2}");
});

test("palette buttons on a reaction: a sign with a new species, an arrow after it, a coefficient, a piece removed", () => {
  const e = new Editor("\\ce{A}");
  const r = reactionOf(e);
  const a = r.body.children[0] as Species;
  e.cursor = { row: a.formula, offset: 1 };
  e.addReactionSign("+");
  // after +, the cursor in the new species' formula; a digit first is its coefficient
  const b = r.body.children[2] as Species;
  assert.equal(e.cursor.row, b.formula);
  e.typeText("2O2");
  assert.equal(e.toLatex(), "\\ce{A + 2 O2}");
  e.addReactionSign("^");
  assert.equal(e.toLatex(), "\\ce{A + 2 O2 ^}");
  // between pieces, after a sign: a new arrow there, the cursor after it
  e.setReactionArrow("->");
  assert.equal(e.toLatex(), "\\ce{A + 2 O2 ^ ->}");
  assert.deepEqual([e.cursor.row, e.cursor.offset], [r.body, 5]);
  e.typeText("B");
  e.enterReactionPlace("coef");
  e.typeText("3");
  assert.equal(e.toLatex(), "\\ce{A + 2 O2 ^ -> 3 B}");
  // the species the cursor is in goes; the cursor where it was
  e.removeReactionPiece();
  assert.equal(e.toLatex(), "\\ce{A + 2 O2 ^ ->}");
  assert.deepEqual([e.cursor.row, e.cursor.offset], [r.body, 5]);
  // between pieces: the one before the cursor goes
  e.removeReactionPiece();
  assert.equal(e.toLatex(), "\\ce{A + 2 O2 ^}");
});

test("an arrow that could still grow (<=> before <=>>) becomes an arrow when something else follows", () => {
  const e = new Editor();
  e.typeText("\\ce A <=> B");
  assert.deepEqual(reactionOf(e).body.children.map((p) => p.kind), ["species", "rxarrow", "species"]);
  assert.equal(e.toLatex(), "\\ce{A <=> B}");
});

test("pieces selected: copied as a reaction of their own, cut, pasted back into the reaction", () => {
  const e = new Editor("\\ce{2 H2 + O2 -> 2 H2O}");
  const r = reactionOf(e);
  e.select(r.body, 0, 3);
  assert.equal(e.copy(), "\\ce{2 H2 + O2}");
  const cut = e.cut();
  assert.equal(cut, "\\ce{2 H2 + O2}");
  assert.equal(e.toLatex(), "\\ce{-> 2 H2O}");
  e.paste(cut);
  assert.equal(e.toLatex(), "\\ce{2 H2 + O2 -> 2 H2O}");
  // other math pasted into a piece: its LaTeX as text, which mhchem reads
  e.cursor = { row: (r.body.children[2] as Species).formula, offset: 2 };
  e.paste("_2");
  assert.equal(e.toLatex(), "\\ce{2 H2 + O2_2 -> 2 H2O}");
});

test("a palette's reaction pieces into a reaction: they replace the pieces selected", () => {
  const e = new Editor("\\ce{A + B}");
  const r = reactionOf(e);
  e.select(r.body, 1, 3);
  e.insertLatex("\\ce{->}");
  assert.equal(e.toLatex(), "\\ce{A ->}");
  // other math inside a piece: its LaTeX as text
  e.cursor = { row: (r.body.children[0] as Species).formula, offset: 1 };
  e.insertLatex("\\frac{#0}{#?}");
  assert.equal(e.toLatex(), "\\ce{A\\frac{}{} ->}");
});

test("what a reaction cannot hold (math between its pieces) is left out of its LaTeX", () => {
  const e = new Editor("\\ce{A}");
  const r = reactionOf(e);
  e.cursor = { row: r.body, offset: 1 };
  e.insertLatex("\\frac{a}{b}");
  assert.deepEqual(r.body.children.map((p) => p.kind), ["species", "frac"]);
  assert.equal(e.toLatex(), "\\ce{A}");
  // and from a selection of pieces put into text, the pieces have no LaTeX alone
  e.select(r.body, 0, 1);
  e.insertLatex("\\text{#0}");
  assert.equal(e.toLatex(), "\\ce{}");
});
