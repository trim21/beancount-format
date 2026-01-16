// @ts-check
const fs = require("fs");
const path = require("path");
const assert = require("assert");
const { createFromBuffer } = require("@dprint/formatter");

const wasmSourcePath = path.join(
  __dirname,
  "target/wasm32-unknown-unknown/release/dprint_plugin_beancount.wasm",
);

function runTests() {
  if (!fs.existsSync(wasmSourcePath)) {
    throw new Error(
      "Missing wasm artifact. Run `cargo build -p dprint-plugin-beancount --target wasm32-unknown-unknown --release` first.",
    );
  }

  const buffer = fs.readFileSync(wasmSourcePath);
  const formatter = createFromBuffer(buffer);
  const result = formatter.formatText({
    filePath: "file.beancount",
    fileText: "2010-01-01 open Assets:Cash",
  });

  assert.strictEqual(result, "2010-01-01 open Assets:Cash\n");
}

runTests()
