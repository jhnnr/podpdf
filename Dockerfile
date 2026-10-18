FROM rust:bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
RUN useradd -m -u 1000 -U podpdf
WORKDIR /home/podpdf
RUN mkdir -p /home/podpdf/data && chown -R podpdf:podpdf /home/podpdf

COPY --from=builder /app/target/release/podpdf /usr/local/bin/podpdf
COPY docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

# Hardened internal container network binding
ENV HOST=0.0.0.0
ENV PORT=8080
EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
