# AEON Web Assembly Module

This is an experimental project which aims to make 
the backend features of [AEON](https://biodivine.fi.muni.cz/aeon) 
available to JavaScript developers through WebAssembly.

**Currently, this should work, but it is all very experimental, so expect
breaking API changes or slow development cadence for the forseeable future.
Most of the algorithms are just barely ported to work somewhat ok.**

### Installing

**TODO: Publish this using NPM.**

### Building

You'll need to have Rust installed (just follow the 
[official guide](https://www.rust-lang.org/learn/get-started) on the Rust 
project website) and then also install `wasm-pack` 
(See the [official installation guide](https://rustwasm.github.io/wasm-pack/installer/)
for the most suitable option for your OS, but running `cargo install wasm-pack` should 
typically do the trick).

Then, you should execute the following:

```
wasm-pack build --release --target web
```

This will produce a `pkg` folder that will contain your hybrid 
JavaScript/WASM `aeon-wasm` package.

## Usage

You can import `aeon-wasm` just like a regular JavaScript module. Then, the following
features become available. You can test these in the `example` folder as well.

### Model format conversions

```JavaScript

```