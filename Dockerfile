FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef

WORKDIR /app

RUN apt update && apt install lld clang -y

FROM chef as planner

COPY . .

# Compute a lock-like file for our project
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json

# Build our project dependencies, not our app
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

ENV SQLX_OFFLINE true

RUN cargo build --release --bin email-newsletter

# Runtime Stage
FROM debian:bullseye-slim as runtime

WORKDIR /app

RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/email-newsletter email-newsletter

COPY configuration configuration

ENV RUST_ENV production

ENTRYPOINT [ "email-newsletter" ]