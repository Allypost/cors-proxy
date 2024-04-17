ARG RUST_VERSION='1.77'
ARG RUST_TARGET='x86_64-unknown-linux-musl'
ARG BINARY_NAME='cors-proxy'

ARG APP_FEATURES=''

ARG RUN_USERNAME='app'
ARG RUN_USER_ID='1000'
ARG RUN_GROUP_ID='1000'

##########
# Step 0 #
##########
##
## Setup base image with cargo-chef
##
FROM rust:${RUST_VERSION} AS chef
# `curl` and `bash` are needed for cargo-binstall
# `musl-tools` and `musl-dev` are needed to build app with musl target
RUN apt-get update && apt-get install -y curl bash musl-tools musl-dev cmake
# Install cargo-binstall
RUN curl -L --proto '=https' --tlsv1.2 -sSf 'https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh' | bash
# Install cargo-chef
RUN cargo binstall cargo-chef --locked --no-confirm
# Add proper target and compile flags
ARG RUST_TARGET
RUN rustup target add "${RUST_TARGET}"
ENV RUSTFLAGS='-C target-feature=+crt-static'
WORKDIR /app


##########
# Step 1 #
##########
##
## Generate a build plan for rust dependencies
##
FROM chef AS planner
COPY . .
# Generate "lockfile" aka dependency dump
RUN cargo chef prepare \
    --recipe-path recipe.json


##########
# Step 2 #
##########
##
## Build app with the cached dependencies
##
FROM chef AS builder
COPY --from=planner /app/recipe.json .
# Build dependencies
ARG RUST_TARGET
ARG APP_FEATURES
ARG BINARY_NAME
RUN cargo chef cook \
    --release \
    --target "${RUST_TARGET}" \
    --features "${APP_FEATURES}" \
    --package "${BINARY_NAME}" \
    --recipe-path recipe.json
# Copy rest of files and compile
# only the remaining app code
COPY . .
ARG RUST_TARGET
ARG APP_FEATURES
ARG BINARY_NAME
RUN cargo build \
    --release \
    --target "${RUST_TARGET}" \
    --features "${APP_FEATURES}" \
    --package "${BINARY_NAME}"


##########
# Step 3 #
##########
##
## Run the app in a configured environment
##
FROM alpine:3 as runner
ARG RUN_USERNAME
ARG RUN_USER_ID
ARG RUN_GROUP_ID
RUN addgroup -g "${RUN_GROUP_ID}" "${RUN_USERNAME}"
RUN adduser -u "${RUN_USER_ID}" -D -G "${RUN_USERNAME}" "${RUN_USERNAME}"
# Install app
ARG RUST_TARGET
ARG BINARY_NAME
COPY --from=builder "/app/target/${RUST_TARGET}/release/${BINARY_NAME}" /usr/local/bin/
RUN chmod a=rx "/usr/local/bin/${BINARY_NAME}"
# Run app
RUN echo -e "#!/bin/sh\n\n/usr/local/bin/${BINARY_NAME} \"\$@\"" > /entrypoint.sh && chmod +x /entrypoint.sh
USER ${RUN_USERNAME}
LABEL org.opencontainers.image.authors="me@allypost.net"
EXPOSE 8000
ENTRYPOINT [ "/entrypoint.sh" ]
CMD [ "--help" ]
