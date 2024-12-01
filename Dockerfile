ARG RUST_VER=latest
FROM rust:${RUST_VER} AS builder
ARG BUILD_MODE=release
ARG BUILD_ARGS=--release

COPY application/src /project/application/src
COPY application/Cargo.toml /project/application/Cargo.toml

COPY common/src /project/common/src
COPY common/Cargo.toml /project/common/Cargo.toml

COPY service/src /project/service/src
COPY service/Cargo.toml /project/service/Cargo.toml

COPY Cargo.toml /project/Cargo.toml
COPY Cargo.lock /project/Cargo.lock

COPY migrations /project/migrations

WORKDIR /project

RUN cargo build ${BUILD_ARGS} --bin application
RUN mv /project/target/${BUILD_MODE}/application /project/out

FROM debian:stable-slim AS runner

COPY --from=builder /project/out /usr/local/bin/application

ENTRYPOINT ["/usr/local/bin/application"]