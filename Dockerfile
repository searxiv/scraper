# Builder ######################################################################

FROM rust:1.76-bookworm as builder

RUN apt-get update && \
  apt-get install -y --no-install-recommends libpoppler-glib-dev=22.12.0-2+b1

WORKDIR /searxiv
COPY Cargo* .
COPY src src

RUN cargo build --release

# Runtime ######################################################################

FROM debian:bookworm-slim AS runtime

RUN apt-get update && \
  apt-get install -y --no-install-recommends libpoppler-glib8=22.12.0-2+b1 \
    libcairo-gobject2=1.16.0-7 && \
  rm -rf /var/lib/apt/lists/*

COPY --from=builder /searxiv/target/release/scraper /scraper

ENTRYPOINT ["/scraper"]

