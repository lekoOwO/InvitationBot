FROM rust:1.75-slim-bookworm as builder

WORKDIR /usr/src/app
COPY . .

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

RUN cargo build --release

# Use distroless as runtime image
FROM gcr.io/distroless/cc-debian12

WORKDIR /app

COPY --from=builder /usr/src/app/target/release/invitationbot /app/
COPY --from=builder /usr/src/app/migrations /app/migrations

ENV DATABASE_URL=sqlite:data/bot.db

VOLUME ["/app/data"]
USER nonroot
ENTRYPOINT ["/app/invitationbot"] 