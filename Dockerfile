# select build image
FROM rust:1.44 as build

RUN /bin/sh -c set -ex;apt-get update;DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends cmake

# create a new empty shell project
RUN USER=root cargo new --bin taichi-agent
WORKDIR /taichi-agent

# copy over your manifests
COPY ./.cargo ./.cargo
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

# this build step will cache your dependencies
RUN cargo build --release
RUN rm ./src/*.rs

# copy your source tree
COPY ./src ./src
COPY ./proto ./proto
COPY ./build.rs ./build.rs

# build for release
RUN rm ./target/release/deps/taichi_agent*
RUN rustup component add rustfmt
RUN cargo build --release

# our final base
FROM rust:1.44

# copy the build artifact from the build stage
COPY --from=build /taichi-agent/target/release/taichi-agent .

# set the startup command to run your binary
CMD ["./taichi-agent"]