# Meterbus Wired Data-Link Node Bindings

Internal napi-rs bindings for the `meterbus-wired-datalink` crate.

This crate generates the native addon used by
[`@meterbus/wired-datalink-node`](../../packages/meterbus-wired-datalink-node/main/).
It is part of the workspace build and is not published to crates.io.

From the workspace root, build and test the generated package with:

```sh
pnpm --filter @meterbus/wired-datalink-node build
pnpm --filter @meterbus/wired-datalink-node test
```

## License

Copyright © 2026 Nicolas HEDGER.

Licensed under either of:

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

at your option.
