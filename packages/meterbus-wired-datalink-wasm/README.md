# Meterbus Wired Data-Link for WebAssembly

[![npm][npm-badge]][npm-url]
[![License][license-badge]][license-url]
[![WebAssembly][wasm-badge]][installation-url]
[![ESM][esm-badge]][installation-url]

Wired M-Bus data-link frames and codecs for WebAssembly.

This package provides typed, validated frame construction plus exact and
streaming decoders for the wired M-Bus link layer defined by
[EN 13757-2:2018+A1:2023](https://www.evs.ee/en/evs-en-13757-2-2018). It uses
browser-native WebAssembly generated from the
[`meterbus-wired-datalink`](https://github.com/nhedger/meterbus/tree/main/crates/meterbus-wired-datalink)
Rust crate with wasm-bindgen.

## Highlights

- ACK, NACK, short, control, and long frame formats
- Exact decoding for complete frames
- Incremental decoding across arbitrary byte chunks
- Strict and noise-resynchronizing stream recovery
- Typed `Frame`, `StreamDecoder`, and recovery results
- `Uint8Array` input and output
- Browser-native WebAssembly without Node-API emulation
- ESM-only package with generated TypeScript declarations
- Browser ESM TypeScript [`examples`](examples/)

## Installation

Add the package with pnpm:

```sh
pnpm add @meterbus/wired-datalink-wasm
```

Initialize the WebAssembly module before using its exports.

## Usage

Construct, encode, and decode a REQ-UD2 short frame:

```js
import init, { Frame } from "@meterbus/wired-datalink-wasm";

await init();

const request = Frame.short(0x5b, 1);
const encoded = request.encode();
const decoded = Frame.decode(encoded);

console.log(decoded.kind); // "short"
```

Decode frames incrementally:

```js
import init, {
  StreamDecoder,
} from "@meterbus/wired-datalink-wasm";

await init();

const decoder = new StreamDecoder();
decoder.push(Uint8Array.of(0x10, 0x5b));

const result = decoder.push(Uint8Array.of(0x01, 0x5c, 0x16));
console.log(result.frames[0].kind); // "short"
```

## Errors

Every data-link failure has a concrete error type with fields describing the
invalid input. All concrete errors extend `DatalinkError`.

```js
import { Frame, UnknownStartByteError } from "@meterbus/wired-datalink-wasm";

try {
  Frame.decode(Uint8Array.of(0xff));
} catch (error) {
  if (error instanceof UnknownStartByteError) {
    console.log(error.actual); // 255
  }
}
```

## License

Copyright © 2026 Nicolas HEDGER.

Licensed under either of:

- [Apache License, Version 2.0][apache-license]
- [MIT License][mit-license]

at your option.

[npm-badge]: https://img.shields.io/npm/v/@meterbus/wired-datalink-wasm.svg?style=flat-square&labelColor=black&color=fed7aa
[npm-url]: https://www.npmjs.com/package/@meterbus/wired-datalink-wasm
[license-badge]: https://img.shields.io/npm/l/@meterbus/wired-datalink-wasm.svg?style=flat-square&labelColor=black&color=bbf7d0
[license-url]: #license
[wasm-badge]: https://img.shields.io/badge/WebAssembly-browser-fed7aa?style=flat-square&labelColor=black
[esm-badge]: https://img.shields.io/badge/modules-ESM-fed7aa?style=flat-square&labelColor=black
[installation-url]: #installation
[apache-license]: https://github.com/nhedger/meterbus/blob/main/LICENSE-APACHE
[mit-license]: https://github.com/nhedger/meterbus/blob/main/LICENSE-MIT
