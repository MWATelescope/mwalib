# Python tests

This directory is for tests relating to the python bindings.

## Instructions

* Run `cargo test` first to get the test_files generated (and to ensure the Rust library is working)

```bash
cargo test --features=cfitsio-static,exmaples --strip
```

* Build from the root `mwalib` directory using:

```bash
maturin develop --features=python,cfitsio-static --strip
```

* Install prerequisites

```bash
pip install -r requirements.txt
```

* Run tests

```bash
pytest
```
