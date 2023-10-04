#
# This image is to provide us with a stable, old platform to build our
# pypi wheels with so we don't rely on newer glibc or other dependencies.
#
# Build this with:
#     docker build . -t pypi_deploy:latest
#
# Run this with: 
#     docker run -it --rm -v $(pwd)/../..:/io pypi_deploy:latest
#
# Once at the prompt, build with:
#     maturin build --release --cargo-extra-args="--features python,cfitsio-static,examples" --strip --manylinux 2014
#
# Publish to the test pypi repository first, with (where PYPI_TOKEN is a token generated from https://test.pypi.org):
#     maturin publish --cargo-extra-args="--features python,cfitsio-static,examples" --manylinux 2014 --username __token__ --password PYPI_TOKEN --repository-url https://test.pypi.org/legacy/
#
# Publish to the real pypi repository, with (where USERNAME and PASSWORD are valid):
#     maturin publish --cargo-extra-args="--features python,cfitsio-static,examples" --manylinux 2014 --username USERNAME --password PASSWORD
#
FROM konstin2/maturin

WORKDIR /io

RUN rustup install 1.63 --no-self-update

RUN rustup default 1.63

RUN rm -rf /io/target

ENTRYPOINT ["/bin/bash"]