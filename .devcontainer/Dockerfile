# mold linker doesn't support rustlang/rust:nightly - requires Debian. Using rust:nightly would require lld which is worse for performance.
FROM rust:1.76-bookworm

ARG DEBIAN_FRONTEND=noninteractive

# xorg is older way of piping viewport to host machine, weston is Wayland (newer version).
# either way, need to have server installed on host machine to receive output
# I tested GWSL and it worked out of the box on Windows - https://opticos.github.io/gwsl/

# clang is needed to compile Rust more quickly
# lld and mold are both Rust linkers. mold is faster, but does not support WASM (https://github.com/rui314/mold/issues/555)
# mold and (xorg or weston) are needed for native builds.
RUN apt-get update && \
   apt-get install -y libasound2-dev libudev-dev clang lld mold xorg

# Need to manually ensure nightly is configured when using bookworm
RUN rustup toolchain install nightly-2024-02-01
RUN rustup component add rustfmt
RUN rustup target install wasm32-unknown-unknown

# trunk is used for serving WASM builds in development
# lock trunk version because they shipped a broken build in v0.18.0, trust has been lost.
RUN cargo install trunk --version 0.18.2
# cargo-watch is used for non-WASM build auto-reload
RUN cargo install cargo-watch

# Ensure hardware acceleration works for native builds when running on Windows host machine
ENV LD_LIBRARY_PATH=/usr/lib/wsl/lib
RUN apt update && apt install -y vainfo mesa-va-drivers
ENV LIBVA_DRIVER_NAME=d3d12

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
