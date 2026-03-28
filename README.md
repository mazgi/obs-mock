# obs-mock

An OBS WebSocket (v5.x) mock server for testing OBS client applications without running OBS Studio.

## Install

Download a prebuilt binary from [GitHub Releases](https://github.com/mazgi/obs-mock/releases), or run with Docker:

```shell
docker compose up -d
```

## Usage

Run the binary directly:

```shell
obs-mock
```

The mock server starts on `ws://localhost:4455` (the default OBS WebSocket port).

### Configuration

Environment variables:

| Variable | Default | Description |
|---|---|---|
| `OBS_MOCK_PORT` | `4455` | WebSocket listen port |
| `OBS_MOCK_PASSWORD` | (none) | Enable authentication with this password |

### Demo scenes

The mock server starts with 5 pre-configured scenes:

| Scene | Items |
|---|---|
| **Main** | Camera, Microphone, Desktop Audio |
| **Screen Share** | Screen Capture, Webcam Overlay (PiP), Desktop Audio |
| **BRB** | BRB Image, Background Music |
| **Starting Soon** | Starting Soon Image, Countdown Timer, Background Music |
| **Ending** | Ending Image, Background Music |

Scenes and their items are fully stateful — you can create, remove, rename, and reorder them via the WebSocket API.

### Supported features

- Full obs-websocket 5.x handshake (Hello/Identify/Identified) with optional SHA256 authentication
- WebSocket subprotocol negotiation (`obswebsocket.json` / `obswebsocket.msgpack`)
- All 114 request types from the protocol, including:
  - Scenes: list, create, remove, rename, switch program/preview
  - Scene Items: list, create, remove, duplicate, transform, enable/disable, lock, reorder
  - Inputs: list, create, remove, mute, volume, settings
  - Streaming: start, stop, toggle, status
  - Recording: start, stop, pause, resume, status
  - Outputs: virtual cam, replay buffer
  - Transitions, Filters, Media Inputs, UI, Config
- Request batching (OpCode 8) with `haltOnFailure` support
- Reidentify (OpCode 3) for updating event subscriptions
- Stateful mock: scenes, scene items, inputs, streaming/recording status persist across requests within a session

## Development

### Prerequisites

- Docker
- Docker Compose

### Setup

Set your host UID and GID so that files created inside the container have correct ownership:

```shell
export UID=$(id -u)
export GID=$(id -g)
```

Build the container:

```shell
docker compose build
```

Start the development container (auto-reloads on code changes):

```shell
docker compose up -d
```

Open a shell inside the container:

```shell
docker compose exec obs-mock bash
```

Stop the container:

```shell
docker compose down
```

### Releasing

Releases are managed by [cargo-dist](https://opensource.axo.dev/cargo-dist/). The release workflow automatically builds binaries for the following platforms:

| OS | Architecture |
|---|---|
| Linux | x86_64, aarch64 |
| macOS | x86_64, aarch64 (Apple Silicon) |
| Windows | x86_64 |

To create a new release:

1. Bump the version in `obs-mock/Cargo.toml`
2. Commit the change
3. Create and push a version tag:

```shell
git tag v0.x.y
git push origin v0.x.y
```

The GitHub Actions workflow will build the binaries and create a GitHub Release automatically.
