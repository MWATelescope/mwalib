# mwalib python pyi stub generation

## How it works

1. Build mwalib using: `cargo build --all-features`
2. Run the stub generator: `target/debug/stub_gen`
3. At this point you can view the `mwalib.pyi` file and see what Python stubs were generated.
4. To test, build a python wheel to test with: `maturin build --all-features --out dist`.
5. In a fresh Python environment install the wheel: `pip install dist/mwalib-1.7.0-cp313-cp313-manylinux_2_34_x86_64.whl`.
6. Open a python file in your IDE and hopefully you have some type info and some doc strings.
