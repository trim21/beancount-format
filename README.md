# @dprint/beancount

This package is kept in-repo to exercise dprint integration tests. It is not
published independently; it assumes you have built the wasm artifact locally.

Use this with [@dprint/formatter](https://github.com/dprint/js-formatter) or
just use @dprint/formatter and download the
[dprint-plugin-beancount WASM file](https://github.com/dprint/dprint-plugin-beancount/releases).

## Running tests locally

```sh
cargo build -p dprint-plugin-beancount --target wasm32-unknown-unknown --features wasm --release
pnpm install --frozen-lockfile
pnpm test
```

## Example

```ts
import { createFromBuffer } from "@dprint/formatter";
import { getPath } from "@dprint/beancount";
import * as fs from "fs";

const buffer = fs.readFileSync(getPath());
const formatter = createFromBuffer(buffer);

console.log(formatter.formatText("example.beancount", "2010-01-01 open Assets:Cash"));
```
