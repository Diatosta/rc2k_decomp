# The Steam Deck's build (and any Linux x86-64 system with glibc 2.35 or later): Ubuntu 22.04, the
# oldest glibc of the SteamOS 3 line and of the current desktop distributions. SDL is built from
# source inside it with its desktop video, sound and input backends, so the one program needs only
# the C library. Rust comes from the host's ~/.cargo and ~/.rustup (tools/deck/run.sh).
FROM ubuntu:22.04
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl xz-utils make cmake ninja-build \
      pkg-config git gnupg gcc g++ binutils clang libclang-dev \
      libx11-dev libxext-dev libxrandr-dev libxcursor-dev libxfixes-dev libxi-dev libxss-dev \
      libxtst-dev libxkbcommon-dev libwayland-dev wayland-protocols libdecor-0-dev \
      libegl-dev libgl-dev libgles-dev libdrm-dev libgbm-dev libvulkan-dev \
      libasound2-dev libpulse-dev libpipewire-0.3-dev libdbus-1-dev libudev-dev \
 && rm -rf /var/lib/apt/lists/*
