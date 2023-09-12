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
# Publish with:
#
#
FROM konstin2/maturin

WORKDIR /
ADD https://github.com/Kitware/CMake/releases/download/v3.19.1/cmake-3.19.1.tar.gz .
RUN tar -xf cmake-3.19.1.tar.gz && \
    cd cmake-3.19.1 && \
    ./bootstrap -- -DCMAKE_USE_OPENSSL=OFF && \
    make -j8 install && \
    cd .. && \
    rm -r cmake*

WORKDIR /io

RUN rustup install 1.63 --no-self-update

RUN rustup default 1.63

RUN rm -rf /io/target

RUN cargo update

ENTRYPOINT ["/bin/bash"]