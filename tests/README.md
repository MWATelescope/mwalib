# Python tests

This directory is for tests relating to the python bindings.

## Instructions

* Build from the root `mwalib` directory using:

```bash
maturin develop -b pyo3 --features=python --strip
```

* Change to the `tests` directory:

```bash
cd tests
```

* Install prerequisites

```bash
pip install -r requirements.txt
```

* Run tests

```bash
pytest
```
