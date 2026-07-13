# Meterbus Wired Data-Link Python Bindings

Internal PyO3 bindings for the `meterbus-wired-datalink` crate.

This crate generates the extension module used by
[`meterbus-wired-datalink`](../../packages/meterbus-wired-datalink-python/).
It is part of the workspace build and is not published to crates.io.

From the workspace root, build and test the Python package with:

```sh
uv run --package meterbus-wired-datalink maturin develop
uv run --package meterbus-wired-datalink pytest
```

## License

Copyright © 2026 Nicolas HEDGER.

Licensed under either of:

- [Apache License, Version 2.0](../../LICENSE-APACHE)
- [MIT License](../../LICENSE-MIT)

at your option.
