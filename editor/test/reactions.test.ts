// Reactions (\ce) edited as pieces: species (coefficient, formula, state), arrows with conditions, signs.

import { test } from "node:test";
import assert from "node:assert/strict";
import { Editor, type Reaction } from "../src/model.ts";
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
