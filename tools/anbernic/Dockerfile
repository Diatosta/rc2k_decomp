# The Anbernic RG34XXSP's build (stock OS: Ubuntu 22.04 arm64, glibc 2.35): the same system's
# cross compiler and ALSA headers. Rust comes from the host's ~/.cargo and ~/.rustup.
FROM ubuntu:22.04
RUN dpkg --add-architecture arm64 \
 && sed -i 's/^deb /deb [arch=amd64] /' /etc/apt/sources.list \
 && printf 'deb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports jammy main universe\ndeb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports jammy-updates main universe\n' > /etc/apt/sources.list.d/arm64.list \
 && apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl xz-utils make cmake ninja-build pkg-config git gnupg \
      gcc g++ gcc-aarch64-linux-gnu g++-aarch64-linux-gnu libasound2-dev:arm64 libclang-dev clang
