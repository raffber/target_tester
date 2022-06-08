FROM ubuntu:20.04

ENV TZ=Europe/Zurich RUSTUP_HOME=/rust CARGO_HOME=/cargo PATH=/cargo/bin:/rust/bin:$PATH HOME=/home

RUN set -euxf && \
    ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone && \
    mkdir -p /data /home /rust /cargo && \
    chmod a+rwx /data /home /rust /cargo && \
    cd /root && \
    apt-get update && \
    apt-get install --yes build-essential python3-pip gfortran python3-numpy \
        python3-regex libudev-dev udev pkg-config npm curl mingw-w64 \
        libclang-dev libusb-1.0-0-dev  && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y  && \
    rustup install stable
