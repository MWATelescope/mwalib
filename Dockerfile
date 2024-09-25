FROM python:3.11-bullseye as base

# suppress perl locale errors
ENV LC_ALL=C
# suppress apt-get prompts
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    automake \
    autoconf \
    build-essential \
    pkg-config \
    && \
    apt-get clean all && \
    rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/* && \
    apt-get -y autoremove

# install cfitsio
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

# Get Rust
ARG RUST_VERSION=stable
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

# install python deps for mwalib python
RUN python -m pip install --force-reinstall --no-cache-dir \
    maturin[patchelf]==1.7.2 \
    pip==24.2 \
    numpy==1.24.4

# mount cwd to /mwalib in Docker
ADD . /mwalib
WORKDIR /mwalib

# build python module and examples
ARG MWALIB_FEATURES=cfitsio-static
RUN maturin build --release --features=python,${MWALIB_FEATURES} && \
    python -m pip install $(ls -1 target/wheels/*.whl | tail -n 1) && \
    cargo build --release --examples --features=examples && \
    rm -rf ${CARGO_HOME}/registry
ENV PATH=${PATH}:/mwalib/target/release/examples/

# allow for tests during build
ARG TEST_SHIM
RUN ${TEST_SHIM}
