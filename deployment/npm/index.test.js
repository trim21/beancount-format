// @ts-check
const assert = require("assert");
const createFromBuffer = require("@dprint/formatter").createFromBuffer;
const getPath = require("./index").getPath;

const buffer = require("fs").readFileSync(getPath());
const formatter = createFromBuffer(buffer);
const result = formatter.formatText({
  filePath: "file.beancount",
  fileText: "2010-01-01 open Assets:Cash",
});

assert.strictEqual(result, undefined);
