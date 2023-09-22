# NOTE: mold linker doesn't support rustlang/rust:nightly - requires Debian. Swapping to rust:nightly would require using lld linker.
# NOTE: mold does not support wasm https://github.com/rui314/mold/issues/555
FROM rust:1.71-bookworm
# FROM rustlang/rust:nightly

# xorg is older way of piping viewport to host machine, weston is Wayland (newer version).
# either way, need to have server installed on host machine to receive output
# I tested GWSL and it worked out of the box on Windows - https://opticos.github.io/gwsl/

# clang is needed to compile Rust more quickly
# lld and mold are both Rust linkers - one is used for linux and other is used for non-linux
RUN apt-get update && \
  apt-get install -y libasound2-dev libudev-dev clang lld

# these are needed for native builds
# mold xorg weston

# Need to manually ensure nightly is configured when using bookworm
RUN rustup toolchain install nightly
RUN rustup default nightly
RUN rustup component add rustfmt
RUN rustup target install wasm32-unknown-unknown

# trunk is used for serving in development and wasm-bindgen-cli is used for testing
RUN cargo install trunk wasm-bindgen-cli

# TODO: If multiple people start working on this code then need a more robust location for this.
# If these aren't set then they need to be set every time the Docker container is rebuilt which is annoying.
RUN git config --global user.name "Sean Anderson"
RUN git config --global user.email "Sean.Palmer.Anderson@gmail.com"

# TODO: If/when I take testing seriously again I can re-enable this logic, but it *DOMINATES* (100-1) Docker setup time
# Install headless chrome driver to support testing (log messages break when using node)
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
