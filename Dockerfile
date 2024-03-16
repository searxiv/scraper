# Builder ######################################################################

FROM rust:1.76-bookworm as builder

WORKDIR /searxiv
COPY Cargo* .
COPY src src

RUN cargo build --release

# Runtime ######################################################################

FROM debian:bookworm-slim AS runtime

COPY --from=builder /searxiv/target/release/scraper /scraper
COPY .env /.env

ENTRYPOINT ["/scraper"]

