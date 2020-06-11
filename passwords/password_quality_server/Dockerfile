FROM rust:1.44.0 as build
RUN apt-get update && apt-get install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/src
RUN USER=root cargo new password_quality_server
WORKDIR /usr/src/password_quality_server
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release --target x86_64-unknown-linux-musl

COPY src ./src
RUN cargo install --target x86_64-unknown-linux-musl --path .

FROM scratch
COPY --from=build /usr/local/cargo/bin/password_quality_server .
USER 1000
EXPOSE 3030
CMD ["./password_quality_server"]
