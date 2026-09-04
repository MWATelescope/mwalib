# syntax=docker/dockerfile:1

# --------------------------------------------------------------------------
# Multi-stage build:
#   builder  - has the full toolchain (rustc/cargo, cmake, build-essential,
#              cfitsio -dev headers) needed to compile mwalib, run its tests,
#              and build the python wheel + venv.
#   runtime  - a clean python:3.13-slim-trixie image that copies over only
#              the finished artifacts (venv, compiled cfitsio .so, example
#              binaries) - no compilers, headers, or Rust toolchain.
# Result: a much smaller, lower-attack-surface image; the builder stage is
# discarded entirely except for what's explicitly copied with --from=builder.
# --------------------------------------------------------------------------

# ---------- builder ----------
FROM python:3.13-slim-trixie AS builder
# Compiles everything (rust, cfitsio, python wheel) - discarded except for
# what the runtime stage below copies out with --from=builder.

# suppress perl locale errors
ENV LC_ALL=C
# suppress apt-get prompts (build-time only, not left set in the final image)
ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    build-essential \
    cmake \
    pkg-config \
    curl \
    libcurl4-openssl-dev \
    zlib1g-dev \
    && apt-get autoclean \
    && apt-get clean \
    && apt-get autoremove -y \
    && rm -rf /var/lib/apt/lists/*

# install cfitsio into /usr/local
ARG CFITSIO_VERSION=4.6.3
RUN curl -fsSL "https://heasarc.gsfc.nasa.gov/FTP/software/fitsio/c/cfitsio-${CFITSIO_VERSION}.tar.gz" -o /tmp/cfitsio.tar.gz && \
    tar -xf /tmp/cfitsio.tar.gz -C /tmp && \
    rm /tmp/cfitsio.tar.gz && \
    cd /tmp/cfitsio-${CFITSIO_VERSION} && \
    cmake -S . -B build \
        -DCMAKE_INSTALL_PREFIX=/usr/local \
        -DUSE_PTHREADS=ON \
        -DUSE_SSE2=OFF \
        -DUSE_SSSE3=OFF \
        -DUSE_CURL=ON && \
    cmake --build build -j && \
    cmake --install build && \
    ldconfig && \
    rm -rf /tmp/cfitsio-${CFITSIO_VERSION}

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

# isolated venv for the python side, so it's a clean, self-contained thing
# to copy into the runtime stage
RUN python -m venv /opt/venv
ENV PATH="/opt/venv/bin:${PATH}"
RUN pip install --no-cache-dir --upgrade pip

# copy source into /mwalib
COPY . /mwalib
WORKDIR /mwalib

# NB: this intentionally ignores Cargo.lock pins in favour of the newest
# compatible deps. Drop this RUN if you want reproducible builds instead.
RUN cargo update --verbose

# build + test the rust examples
RUN cargo build --features=examples && \
    cargo test --features=examples

# Build and install the python module + its "dev" dependency group straight
# from pyproject.toml:
#   - maturin's version comes from [build-system].requires (maturin>=1.0,<2.0)
#   - the "python"/"pyo3-extension-module" features come from [tool.maturin]
#   - numpy comes along automatically as a declared [project] dependency
#   - pytest/ruff/toml/ty come from the "dev" group under [dependency-groups]
#     (PEP 735 - needs pip>=25.1, hence the upgrade above)
RUN pip install --no-cache-dir . --group dev

RUN pytest


# ---------- runtime ----------
FROM python:3.13-slim-trixie AS runtime
# Clean image: only runtime shared libs + the built venv/binaries from
# `builder` - no compilers or dev headers ship in the final image.

ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    libcurl4 \
    zlib1g \
    && apt-get autoclean \
    && apt-get clean \
    && apt-get autoremove -y \
    && rm -rf /var/lib/apt/lists/*

# cfitsio's runtime shared library, built in the builder stage (root, so it
# can update the system linker cache)
COPY --from=builder /usr/local/lib/libcfitsio* /usr/local/lib/
RUN ldconfig

# optional: run as a non-root user now that build tooling is gone.
# Drop these two lines (and the --chown flags below) to keep running as root.
RUN useradd --create-home --shell /bin/bash mwalib
USER mwalib

# the venv with mwalib + numpy already installed
COPY --from=builder --chown=mwalib:mwalib /opt/venv /opt/venv
ENV PATH="/opt/venv/bin:${PATH}"

# source tree + compiled rust example binaries
WORKDIR /mwalib
COPY --chown=mwalib:mwalib . /mwalib
COPY --from=builder --chown=mwalib:mwalib /mwalib/target/debug/examples /mwalib/target/debug/examples

RUN python -c "import sys; print(f'{sys.implementation=}')"