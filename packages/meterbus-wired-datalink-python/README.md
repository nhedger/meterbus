# Meterbus Wired Data-Link for Python

[![PyPI][pypi-badge]][pypi-url]
[![License][license-badge]][license-url]
[![Python 3.10+][python-badge]][installation-url]
[![Typed][typed-badge]][installation-url]

Native wired M-Bus data-link frames and codecs for Python.

This package provides typed, validated frame construction plus exact and
streaming decoders. It uses a native extension generated from the
[`meterbus-wired-datalink`](https://github.com/nhedger/meterbus/tree/main/crates/meterbus-wired-datalink)
Rust crate with PyO3.

## Highlights

- ACK, NACK, short, control, and long frame formats
- Exact decoding for complete frames
- Incremental decoding across arbitrary byte chunks
- Strict and noise-resynchronizing stream recovery
- Typed `Frame`, `StreamDecoder`, and recovery results
- Python `bytes` input and output
- Stable ABI wheels for Python 3.10 and newer
- Type information through `py.typed` and `.pyi` stubs
- Python [`examples`](examples/)

## Installation

Add the package with uv:

```sh
uv add meterbus-wired-datalink
```

Python 3.10 or newer is required.

## Usage

Construct, encode, and decode a REQ-UD2 short frame:

```python
from meterbus_wired_datalink import Frame

request = Frame.short(0x5B, 1)
encoded = request.encode()
decoded = Frame.decode(encoded)

print(decoded.kind)  # "short"
```

Decode frames incrementally:

```python
from meterbus_wired_datalink import StreamDecoder

decoder = StreamDecoder()
decoder.push(bytes.fromhex("105b"))

result = decoder.push(bytes.fromhex("015c16"))
print(result.frames[0].kind)  # "short"
```

## Errors

Every data-link failure has a concrete exception type with attributes
describing the invalid input. All concrete exceptions extend `DatalinkError`.

```python
from meterbus_wired_datalink import Frame, UnknownStartByteError

try:
    Frame.decode(b"\xff")
except UnknownStartByteError as error:
    print(error.actual)  # 255
```

## License

Copyright © 2026 Nicolas HEDGER.

Licensed under either of:

- [Apache License, Version 2.0][apache-license]
- [MIT License][mit-license]

at your option.

[pypi-badge]: https://img.shields.io/pypi/v/meterbus-wired-datalink.svg?style=flat-square&labelColor=black&color=fed7aa
[pypi-url]: https://pypi.org/project/meterbus-wired-datalink/
[license-badge]: https://img.shields.io/pypi/l/meterbus-wired-datalink.svg?style=flat-square&labelColor=black&color=bbf7d0
[license-url]: #license
[python-badge]: https://img.shields.io/badge/Python-3.10%2B-fed7aa?style=flat-square&labelColor=black
[typed-badge]: https://img.shields.io/badge/types-py.typed-fed7aa?style=flat-square&labelColor=black
[installation-url]: #installation
[apache-license]: https://github.com/nhedger/meterbus/blob/main/LICENSE-APACHE
[mit-license]: https://github.com/nhedger/meterbus/blob/main/LICENSE-MIT
