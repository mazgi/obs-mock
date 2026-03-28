# obs-mock

An OBS WebSocket (v5.x) mock server for testing OBS client applications without running OBS Studio.

## Prerequisites

- Docker
- Docker Compose

## Setup

Set your host UID and GID so that files created inside the container have correct ownership:

```shell
export UID=$(id -u)
export GID=$(id -g)
```

Build the container:

```shell
docker compose build
```

## Run

Start the container:

```shell
docker compose up -d
```

Open a shell inside the container:

```shell
docker compose exec obs-mock bash
```

Build and run the application:

```shell
cargo run
```

The mock server starts on `ws://localhost:4455` (the default OBS WebSocket port).

### Configuration

Environment variables:

| Variable | Default | Description |
|---|---|---|
| `OBS_MOCK_PORT` | `4455` | WebSocket listen port |
| `OBS_MOCK_PASSWORD` | (none) | Enable authentication with this password |

### Supported features

- Full obs-websocket 5.x handshake (Hello/Identify/Identified) with optional SHA256 authentication
- All 114 request types from the protocol, including:
  - Scenes: list, create, remove, rename, switch program/preview
  - Inputs: list, create, remove, mute, volume, settings
  - Streaming: start, stop, toggle, status
  - Recording: start, stop, pause, resume, status
  - Outputs: virtual cam, replay buffer
  - Transitions, Filters, Scene Items, Media Inputs, UI, Config
- Request batching (OpCode 8) with `haltOnFailure` support
- Reidentify (OpCode 3) for updating event subscriptions
- Stateful mock: scenes, inputs, streaming/recording status persist across requests within a session

## Stop

```shell
docker compose down
```
