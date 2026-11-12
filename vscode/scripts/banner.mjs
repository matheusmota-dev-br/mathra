// media/banner.png from docs/vscode-banner.svg (drawn by mathra, `cargo run -p mathra --example banner`),
// at twice its size: the Marketplace takes no SVG in a README, so the extension's README shows this PNG.
// node scripts/banner.mjs

import { readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { initWasm, Resvg } from "@resvg/resvg-wasm";

const at = (path) => new URL(path, import.meta.url);
await initWasm(readFileSync(createRequire(import.meta.url).resolve("@resvg/resvg-wasm/index_bg.wasm")));
const svg = readFileSync(at("../../docs/vscode-banner.svg"), "utf8");
writeFileSync(at("../media/banner.png"), new Resvg(svg, { fitTo: { mode: "width", value: 2560 } }).render().asPng());
console.log("media/banner.png");
