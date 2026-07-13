# Meterbus Wired Data-Link

[![Crates.io][crates-badge]][crates-url]
[![License][license-badge]][license-url]
[![no_std][no-std-badge]][installation-url]
[![no alloc][alloc-badge]][installation-url]
[![MSRV: 1.85][msrv-badge]][installation-url]

Wired M-Bus data-link frames and codecs for Rust.

This crate provides typed, validated frame construction plus
exact and streaming decoders for the wired M-Bus link layer defined by
[EN 13757-2:2018+A1:2023](https://www.evs.ee/en/evs-en-13757-2-2018). The
default build is `no_std` and allocation-free. It is
designed for masters, meters, bridges, embedded devices, and diagnostic tools.

- Typed ACK, NACK, short, control, and long frames
- `no_std`, allocation-free codecs with optional `alloc`
- Exact and incremental stream decoding
- Strict decoding or noise-resynchronizing recovery
- Typed fields and detailed validation errors

Official bindings are available for Node.js, Python, and browser WebAssembly:

[![Node.js 24+][node-bindings-badge]][node-bindings-url] [![Python 3.10+][python-bindings-badge]][python-bindings-url] [![WASM browser][wasm-bindings-badge]][wasm-bindings-url]

## Installation

Add the crate without any features for a `no_std`, allocation-free build:

```toml
[dependencies]
meterbus-wired-datalink = "0.0.3"
```

To use convenience methods that return vectors, enable the optional `alloc`
feature:

```toml
[dependencies]
meterbus-wired-datalink = { version = "0.0.3", features = ["alloc"] }
```

## License

Copyright © 2026 Nicolas HEDGER.

Licensed under either of:

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

at your option.

## Reference material

Development is guided by the following standards:

The current implementation follows the link-layer frame formats,
communication types, addressing rules, and sequencing fields from
[EN 13757-2:2018+A1:2023](https://www.evs.ee/en/evs-en-13757-2-2018). That
standard uses the FT1.2 frame format defined by the
[EN 60870-5 series](https://www.evs.ee/en/evs-en-60870-5-1-2002).

| Document                                                             | Title                        |
| -------------------------------------------------------------------- | ---------------------------- |
| [EN 13757-1:2021](https://www.evs.ee/en/evs-en-13757-1-2021)         | Data exchange                |
| [EN 13757-2:2018+A1:2023](https://www.evs.ee/en/evs-en-13757-2-2018) | Wired M-Bus communication    |
| [EN 13757-3:2025](https://www.evs.ee/en/evs-en-13757-3-2025)         | Application protocols        |
| [EN 60870-5-1](https://www.evs.ee/en/evs-en-60870-5-1-2002)          | Transmission frame formats   |
| [EN 60870-5-2](https://www.evs.ee/en/evs-en-60870-5-2-2002)          | Link transmission procedures |

[crates-badge]: https://img.shields.io/crates/v/meterbus-wired-datalink.svg?style=flat-square&labelColor=black&color=fed7aa
[crates-url]: https://crates.io/crates/meterbus-wired-datalink
[license-badge]: https://img.shields.io/crates/l/meterbus-wired-datalink.svg?style=flat-square&labelColor=black&color=bbf7d0
[license-url]: #license
[no-std-badge]: https://img.shields.io/badge/no__std-compatible-fed7aa?style=flat-square&labelColor=black&logoColor=white
[alloc-badge]: https://img.shields.io/badge/alloc-optional-fed7aa?style=flat-square&labelColor=black
[msrv-badge]: https://img.shields.io/badge/MSRV-1.85-fed7aa?style=flat-square&labelColor=black
[installation-url]: #installation
[node-bindings-badge]: https://img.shields.io/badge/Node.js-24%2B-black?style=flat-square&labelColor=5FA04E&logo=nodedotjs&logoColor=white
[node-bindings-url]: https://github.com/nhedger/meterbus/tree/main/crates/meterbus-wired-datalink-node
[wasm-bindings-badge]: https://img.shields.io/badge/WASM-browser-black?style=flat-square&labelColor=654FF0&logo=webassembly&logoColor=white
[wasm-bindings-url]: https://github.com/nhedger/meterbus/tree/main/crates/meterbus-wired-datalink-wasm
[python-bindings-badge]: https://img.shields.io/badge/Python-3.10%2B-black?style=flat-square&labelColor=3776AB&logo=python&logoColor=white
[python-bindings-url]: https://github.com/nhedger/meterbus/tree/main/crates/meterbus-wired-datalink-python
