reproduce branch for broken tree-sitter wasm32 malloc

step to produce:

```
task build:wasm
```

or

```
export RUSTFLAGS="-C link-args=--max-memory=4294967296 -C link-arg=--initial-heap=104857600"
cargo build -p dprint-plugin-beancount --target wasm32-unknown-unknown --features wasm --release
cp ./target/wasm32-unknown-unknown/release/dprint_plugin_beancount.wasm ./plugin.wasm
```

then install dprint

you can download it from https://github.com/dprint/dprint/releases/tag/0.51.1 or use npm shipped binary, run `pnpm install`

then, run dprint:

```
dprint fmt
```

or

```
pnpm exec dprint fmt
```

It's not a bug that will reproduce 100% in everytimes, you may need to run it multiple times.

what's more:

you can un-comment `[patch.crates-io]` in `./Cargo.toml`, 
there is a fixed version or [tree-sitter-language-0.1.6](https://github.com/trim21/beancount-format/tree/broken-malloc/patches/tree-sitter-language-0.1.6) which contains a c malloc implemented in rust.
