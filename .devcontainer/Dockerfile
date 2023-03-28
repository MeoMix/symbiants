# NOTE: could use rustlang/rust:nightly + lld, but mold linker only ships binaries for Debian 12 (bookworm) which isn't what nightly uses.
# NOTE: unfortunately, mold does not support wasm https://github.com/rui314/mold/issues/555
# FROM rust:1.67-bookworm
FROM rustlang/rust:nightly

RUN apt-get update && \
  apt-get install -y libasound2-dev libudev-dev lld clang

# NOTE: switch to nightly if base image is rust:1.67-bookworm
# RUN rustup toolchain install nightly
# RUN rustup default nightly
RUN rustup target install wasm32-unknown-unknown

# trunk is used for serving in development and wasm-bindgen-cli is used for testing
RUN cargo install trunk wasm-bindgen-cli

# Install headless chrome driver to support testing (log messages break when using node)
# https://gist.github.com/varyonic/dea40abcf3dd891d204ef235c6e8dd79?permalink_comment_id=4368270#gistcomment-4368270
RUN apt-get update && \
    apt-get install -y gnupg wget curl unzip --no-install-recommends && \
    wget -q -O - https://dl-ssl.google.com/linux/linux_signing_key.pub | apt-key add - && \
    echo "deb http://dl.google.com/linux/chrome/deb/ stable main" >> /etc/apt/sources.list.d/google.list && \
    apt-get update -y && \
    apt-get install -y google-chrome-stable && \
    CHROMEVER=$(google-chrome --product-version | grep -o "[^\.]*\.[^\.]*\.[^\.]*") && \
    DRIVERVER=$(curl -s "https://chromedriver.storage.googleapis.com/LATEST_RELEASE_$CHROMEVER") && \
    wget -q --continue -P /chromedriver "http://chromedriver.storage.googleapis.com/$DRIVERVER/chromedriver_linux64.zip" && \
    unzip /chromedriver/chromedriver* -d /chromedriver

ENV CHROMEDRIVER=/chromedriver/chromedriver
