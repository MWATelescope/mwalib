FROM ubuntu:20.04 as base

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
    libpython3-dev \
    python3 \
    python3-dev \
    python3-pip \
    python3-wheel \
    python3-importlib-metadata \
    curl \
    && \
    apt-get clean all && \
    rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/* && \
    apt-get -y autoremove

FROM base AS building
# issues compiling lazy_static on arm64 around mwalib0.15, we're rolling our own Python.
# from this https://github.com/arnaudblois/python-ubuntu-image/blob/main/src/Dockerfile
RUN apt-get update \
    && apt-get install -y --no-install-recommends && apt-get install -y \
    build-essential \
    libbz2-dev \
    libgdbm-dev \
    liblzma-dev \
    libncurses5-dev \
    tzdata \
    zlib1g-dev \
    wget \
    ca-certificates

ARG QUICK_BUILD="false"
ARG PY_VERSION="3.11.0"
# BASE_PYTHON_VERSION: we strip the alpha, beta, rc, etc. For instance 3.11.0rc1 -> 3.11.0
# export BASE_PYTHON_VERSION=`echo ${PY_VERSION} | sed -r "s/([0-9]+\.[0-9]+\.[0-9]+)([a-zA-Z]+[0-9]+)?/\1/"` && \
ARG BASE_PYTHON_VERSION="3.11.0"

RUN echo "Downloading sources"


RUN set +x && \
    wget -c https://www.python.org/ftp/python/${BASE_PYTHON_VERSION}/Python-${PY_VERSION}.tgz --verbose && \
    ls -al && \
    pwd && \
    tar -xzf Python-${PY_VERSION}.tgz; \
    ls -al && \
    if [ "${QUICK_BUILD}" = true ] ; then OPTIMIZATION="" ; else OPTIMIZATION="--enable-optimizations --with-lto"; fi && \
    ls -al && \
    cd Python-${PY_VERSION} && \
    ./configure \
    --with-openssl=/usr/local/ssl \
    --enable-loadable-sqlite-extensions \
    --enable-shared \
    --with-openssl-rpath=auto \
    ${OPTIMIZATION}

RUN echo "Building Python ${PY_VERSION}"
WORKDIR /Python-${PY_VERSION}
# Make sure the env variable are correctly set for Python to be able
# to link and compile against openSSL.
ENV LDFLAGS "-L/usr/local/ssl/lib64/ -Wl,-rpath=/usr/local/ssl/lib64:/usr/local/lib"
ENV LD_LIBRARY_PATH "/usr/local/ssl/lib/:/usr/local/ssl/lib64/"
ENV CPPFLAGS "-I/usr/local/ssl/include -I/usr/local/ssl/include/openssl"


RUN if [ "${QUICK_BUILD}" = true ] ; then OPTIMIZATION="" ; else OPTIMIZATION="--enable-optimizations --with-lto"; fi && \
    ./configure \
    --with-openssl=/usr/local/ssl \
    --enable-loadable-sqlite-extensions \
    --enable-shared \
    --with-openssl-rpath=auto \
    ${OPTIMIZATION}
RUN make --quiet
RUN make --quiet altinstall
# We make sure to remove all the fluff
# recipe taken from the official image https://github.com/docker-library/python/
RUN find /usr/local -depth \
    \( \
    \( -type d -a \( -name test -o -name tests -o -name idle_test \) \) \
    -o \
    \( -type f -a \( -name '*.pyc' -o -name '*.pyo' \) \) \
    \) -exec rm -rf '{}' +;

# back to base
FROM base
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    bzip2 \
    ca-certificates \
    curl \
    libgdbm6 \
    liblzma5 \
    libncurses6 \
    zlib1g

COPY --from=building /usr/local/ /usr/local/

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

# use python3 as the default python
RUN update-alternatives --install /usr/bin/python python /usr/bin/python3 1

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

# # allow for tests during build
# ARG TEST_SHIM="python -c $'print(__import__("sys").implementation, __import__("mwalib").__version__,file="/pyimpl.txt")'"
# RUN ${TEST_SHIM}
