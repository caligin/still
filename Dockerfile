FROM rust:1.45.2-slim-stretch AS build
WORKDIR /

# need nightly for rocket
RUN rustup default nightly && rustup target add x86_64-unknown-linux-musl
# creates dummy files so we can cache the deps layer
RUN USER=root cargo new app
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN cargo build --target x86_64-unknown-linux-musl --release

COPY src ./src
COPY build.rs ./
RUN cargo build --target x86_64-unknown-linux-musl --release

# Copy the statically-linked binary into a scratch container.
FROM scratch
COPY --from=build /app/target/x86_64-unknown-linux-musl/release .
COPY assets ./assets

USER 1000
CMD ["./still"]