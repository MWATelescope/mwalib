FROM python:3.12-slim-bookworm AS base

# suppress perl locale errors
ENV LC_ALL=C
# suppress apt-get prompts
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
    && apt-get install -y --no-install-recommends \    
    build-essential \
    cmake \
    pkg-config \
    curl \
    libcurl4-openssl-dev \
    libz-dev \    
    && apt-get autoclean \
    && apt-get clean \
    && apt-get autoremove -y \
    && rm -rf /var/lib/apt/lists/*
    
# # # install python deps for mwalib python
RUN python -m pip install --force-reinstall --no-cache-dir \
    maturin[patchelf] \
    pip \
    numpy \
    pytest

# # install cfitsio
ARG CFITSIO_VERSION=4.6.3
RUN cd / && \
    curl "https://heasarc.gsfc.nasa.gov/FTP/software/fitsio/c/cfitsio-${CFITSIO_VERSION}.tar.gz" -o cfitsio.tar.gz && \
    tar -xf cfitsio.tar.gz && \
    rm cfitsio.tar.gz && \
    cd cfitsio-${CFITSIO_VERSION} && \
    cmake -S . -B build \
        -DCMAKE_INSTALL_PREFIX=/usr/local \        
        -DUSE_PTHREADS=ON \
        -DUSE_SSE2=OFF \
        -DUSE_SSSE3=OFF \
        -DUSE_CURL=ON && \
    cmake --build build -j && \
    cmake --install build && \
    ldconfig && \
    cd / && \
    rm -rf /cfitsio-${CFITSIO_VERSION}

# # Get Rust
ARG RUST_VERSION=1.90
ENV RUSTUP_HOME=/opt/rust CARGO_HOME=/opt/cargo
ENV PATH="${CARGO_HOME}/bin:${PATH}"
RUN mkdir -m755 $RUSTUP_HOME $CARGO_HOME && ( \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | env RUSTUP_HOME=$RUSTUP_HOME CARGO_HOME=$CARGO_HOME sh -s -- -y \
    --profile=minimal \
    --component llvm-tools \
    --default-toolchain=${RUST_VERSION} \
    )

# mount cwd to /mwalib in Docker
ADD . /mwalib
WORKDIR /mwalib

# Update cargo registry
RUN cargo update --verbose

# build python module and examples
RUN cargo build --examples --features=examples && \
    cargo test --examples --features=examples && \
    maturin build --features=python && \
    python -m pip install $(ls -1 target/wheels/*.whl | tail -n 1) && \
    rm -rf ${CARGO_HOME}/registry
ENV PATH=${PATH}:/mwalib/target/debug/examples/

# Run Python tests
RUN pytest

RUN <<EOF
#!/usr/bin/env python
import sys
from sys import implementation, stdout
print( f"{implementation=}", file=stdout)
EOF