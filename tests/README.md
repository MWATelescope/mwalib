# Python tests

This directory is for tests relating to the python bindings.

## Instructions

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
