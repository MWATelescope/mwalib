# FROM ubuntu:20.04 as base
FROM python:3.11-bookworm as base

# suppress perl locale errors
ENV LC_ALL=C
# suppress apt-get prompts
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    autoconf \
    automake \
    build-essential \
    curl \
    pkg-config \
    && apt-get autoclean \
    && apt-get clean \
    && apt-get autoremove -y \
    && rm -rf /var/lib/apt/lists/*
    # libpython3-dev \
    # python3 \
    # python3-dev \
    # python3-pip \
    # python3-wheel \

# use python3 as the default python
RUN update-alternatives --install /usr/bin/python python /usr/bin/python3 1

# # # install python deps for mwalib python
RUN python -m pip install --force-reinstall --no-cache-dir \
    maturin[patchelf]==1.7.2 \
    pip==24.2 \
    numpy==1.24.4

# # install cfitsio
ARG CFITSIO_VERSION=3.49
RUN cd / && \
    curl "https://heasarc.gsfc.nasa.gov/FTP/software/fitsio/c/cfitsio-${CFITSIO_VERSION}.tar.gz" -o cfitsio.tar.gz && \
    tar -xf cfitsio.tar.gz && \
    rm cfitsio.tar.gz && \
    cd cfitsio-${CFITSIO_VERSION} && \
    ./configure --prefix=/usr/local --enable-reentrant --disable-curl && \
    make shared && \
    make install && \
    ldconfig && \
    cd / && \
    rm -rf /cfitsio-${CFITSIO_VERSION}

# # Get Rust
ARG RUST_VERSION=1.80
ENV RUSTUP_HOME=/opt/rust CARGO_HOME=/opt/cargo
ENV PATH="${CARGO_HOME}/bin:${PATH}"
RUN mkdir -m755 $RUSTUP_HOME $CARGO_HOME && ( \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | env RUSTUP_HOME=$RUSTUP_HOME CARGO_HOME=$CARGO_HOME sh -s -- -y \
    --profile=minimal \
    --component llvm-tools \
    --default-toolchain=${RUST_VERSION} \
    )

# # Get cargo make, llvm-cov (for CI)
# RUN cargo install --force cargo-make cargo-llvm-cov && \
#     rm -rf ${CARGO_HOME}/registry

# mount cwd to /mwalib in Docker
ADD . /mwalib
WORKDIR /mwalib

# build python module and examples
RUN maturin build --verbose --features=python && \
    python -m pip install $(ls -1 target/wheels/*.whl | tail -n 1) && \
    cargo build --verbose --examples --features=examples && \
    rm -rf ${CARGO_HOME}/registry
ENV PATH=${PATH}:/mwalib/target/debug/examples/

RUN <<EOF
#!/usr/bin/env python
import sys
from sys import implementation, stdout
print( f"{implementation=}", file=stdout)
EOF

# allow for tests in CI
# ARG TEST_SHIM="cargo test"
ARG TEST_SHIM=""
RUN ${TEST_SHIM}
