# Meterbus Wired Data-Link WebAssembly Bindings

Internal wasm-bindgen bindings for the `meterbus-wired-datalink` crate.

This crate generates the WebAssembly module used by
[`@meterbus/wired-datalink-wasm`](../../packages/meterbus-wired-datalink-wasm/).
It is part of the workspace build and is not published to crates.io.

From the workspace root, build and test the generated package with:

```sh
pnpm --filter @meterbus/wired-datalink-wasm build
pnpm --filter @meterbus/wired-datalink-wasm test
```

## License

Copyright © 2026 Nicolas HEDGER.

Licensed under either of:

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

at your option.
