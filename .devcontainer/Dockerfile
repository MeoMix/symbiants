# mold linker doesn't support rustlang/rust:nightly - requires Debian. Using rust:nightly would require lld which is worse for performance.
FROM rust:1.80-slim-bookworm

ARG DEBIAN_FRONTEND=noninteractive

# Ensure hardware acceleration works for native builds when running on Windows host machine
ENV LD_LIBRARY_PATH=/usr/lib/wsl/lib
ENV LIBVA_DRIVER_NAME=d3d12

# Need to manually ensure nightly is configured when using bookworm
RUN rustup toolchain install nightly && \
    rustup default nightly && \
    rustup component add rustfmt && \
    rustup target add wasm32-unknown-unknown

# xorg is older way of piping viewport to host machine, weston is Wayland (newer version).
# either way, need to have server installed on host machine to receive output
# I tested GWSL and it worked out of the box on Windows - https://opticos.github.io/gwsl/

# clang is needed to compile Rust more quickly
# lld and mold are both Rust linkers. mold is faster, but does not support WASM (https://github.com/rui314/mold/issues/555)
# mold and (xorg or weston) are needed for native builds.
# vainfo and mesa-va-drivers are used for GPU hardware acceleration when running natively.
RUN apt-get update && \
    apt-get install -y curl git && \
    apt-get install -y libxkbcommon-x11-dev clang mold xorg && \
    apt-get install -y vainfo mesa-va-drivers && \
    rm -rf /var/lib/apt/lists/*

# install pre-compiled cargo-binstall binary
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

# trunk is used for serving WASM builds in development
# cargo-watch is used for non-WASM build auto-reload
RUN cargo binstall --no-confirm trunk@0.20.3 wasm-bindgen-cli@0.2.92 wasm-opt@0.116.1 cargo-watch@8.5.2

# TODO: If/when I take testing seriously again I can re-enable this logic, but installing Chrome Driver increases Docker setup time 100x
# RUN cargo install wasm-bindgen-cli

# https://gist.github.com/varyonic/dea40abcf3dd891d204ef235c6e8dd79?permalink_comment_id=4368270#gistcomment-4368270
# RUN apt-get update && \
#     apt-get install -y gnupg wget curl unzip --no-install-recommends && \
#     wget -q -O - https://dl-ssl.google.com/linux/linux_signing_key.pub | apt-key add - && \
#     echo "deb http://dl.google.com/linux/chrome/deb/ stable main" >> /etc/apt/sources.list.d/google.list && \
#     apt-get update -y && \
#     apt-get install -y google-chrome-stable && \
#     CHROMEVER=$(google-chrome --product-version | grep -o "[^\.]*\.[^\.]*\.[^\.]*") && \
#     DRIVERVER=$(curl -s "https://chromedriver.storage.googleapis.com/LATEST_RELEASE_$CHROMEVER") && \
#     wget -q --continue -P /chromedriver "http://chromedriver.storage.googleapis.com/$DRIVERVER/chromedriver_linux64.zip" && \
#     unzip /chromedriver/chromedriver* -d /chromedriver

# ENV CHROMEDRIVER=/chromedriver/chromedriver
