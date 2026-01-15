const fs = require("fs");
const path = require("path");

const wasmPath = path.join(__dirname, "plugin.wasm");

function getWasmBuffer() {
  return fs.readFileSync(wasmPath);
}

module.exports = {
  wasmPath,
  getWasmBuffer,
};
