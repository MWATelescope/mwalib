# mwalib python pyi stub generation

## How it works

1. Build mwalib using: `cargo build --all-features`
2. Run the stub generator: `target/debug/stub_gen`
3. At this point you can view the `mwalib.pyi` file and see what Python stubs were generated.
4. To test, build a python wheel to test with: `maturin build --all-features --out dist`.
5. In a freah Python environment install the wheel: `pip install dist/mwalib-1.7.0-cp313-cp313-manylinux_2_34_x86_64.whl`.
6. Open a python file in your IDE and hopefully you have some type info and some doc strings.

## Caveats

* Due to [this issue](https://github.com/Jij-Inc/pyo3-stub-gen/issues/93) and [this issue](https://github.com/PyO3/pyo3/issues/780), the codem as of mwalib 1.7.0 does NOT produce a full mwalib.pyi file, due to the fact that the stub generation requires the `python` feature and to get all the struct members to appear in the stub you need to decorate each member with `#[pyo3(get,set)]` but this decorator does not work with the `#[cfg_attr(feature = "python", pyo3(get,set))]` syntax needed to allow mwalib to be compiled without the `python` feature! So to get past this, I have removed the `#[cfg_attr(feature = "python", pyo3(get,set))]` syntax from all struct members and changed `#[cfg_attr(feature = "python", pyo3::pyclass]` on each struct to `#[cfg_attr(feature = "python", pyo3::pyclass(get_all, set_all))]` which will still create the python bindings but none of the struct members will be emitted when generating the stub file!

* Docstrings for `#[new]` methods on structs/classes do not get generated.

* `__enter__` method for a class gets the wrong generated stub so I have to override it (see below).

* Some other manual fixes can be seen in `bin/stubgen.rs`. Hacky but we live in an imperfect world!
