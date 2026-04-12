# Stage 1: Build frontend
FROM node:22-slim AS frontend
WORKDIR /app/frontend
COPY frontend/ ./
RUN npm install && npm run build

# Stage 2: Build backend
FROM rust:1-bookworm AS backend
WORKDIR /app/server
COPY server/Cargo.toml server/Cargo.lock ./
# Pre-build dependencies for caching
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src
COPY server/src ./src
RUN touch src/main.rs && cargo build --release

# Stage 3: Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=backend /app/server/target/release/ironcalc_nextcloud_server ./ironcalc_nextcloud_server
COPY --from=frontend /app/frontend/dist/assets/ ./assets/
COPY server/assets/ironcalc-white.svg server/assets/ironcalc-black.svg ./assets/
COPY server/Rocket.toml ./Rocket.toml

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_SCRIPT_PATH=assets/main

EXPOSE 2620
CMD ["./ironcalc_nextcloud_server"]
