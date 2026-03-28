# obs-mock

OBS WebSocket (v5.x) mock server written in Rust.

## Project structure

- `obs-mock/` — Rust application (Cargo.toml, src/)
- `Dockerfiles.d/obs-mock/` — Dockerfile for development container
- `compose.yaml` — Docker Compose config

## Development

- Run inside Docker: `docker compose exec obs-mock bash`, then `cargo run`
- Set `UID=$(id -u)` and `GID=$(id -g)` before `docker compose build`
