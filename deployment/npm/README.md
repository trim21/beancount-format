# @dprint/beancount

npm distribution of [dprint-plugin-beancount](https://github.com/dprint/dprint-plugin-beancount).

Use this with [@dprint/formatter](https://github.com/dprint/js-formatter) or just use @dprint/formatter and download the [dprint-plugin-beancount WASM file](https://github.com/dprint/dprint-plugin-beancount/releases).

## Example

```ts
import { createFromBuffer } from "@dprint/formatter";
import { getPath } from "@dprint/beancount";
import * as fs from "fs";

const buffer = fs.readFileSync(getPath());
const formatter = createFromBuffer(buffer);

console.log(formatter.formatText("example.beancount", "2010-01-01 open Assets:Cash"));
```
