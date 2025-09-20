# WICTK – Weather & Lightning Tracking System

## Table of Contents
- [Overview](#overview)
- [Architecture](#architecture)
- [Project Structure](#project-structure)
- [Core Components](#core-components)
- [Development Guidelines](#development-guidelines)
- [Build, Test, and Run](#build-test-and-run)
- [Contributing](#contributing)
- [License](#license)
- [Contact](#contact)

---

## Overview

**WICTK** is a modular, multi-crate Rust workspace for real-time weather and lightning tracking, alerting, and logging. It is designed for reliability, extensibility, and ease of deployment, with a focus on robust backend services, shared business logic, and client-side logging.

---

## Architecture

```
+-------------------+      +-------------------+      +-------------------+
|   client_logger   | ---> |     backend       | ---> |    wictk_core     |
+-------------------+      +-------------------+      +-------------------+
        |                        |                           |
        |                        |                           |
        |                        v                           v
        |                [External APIs]             [Database/Storage]
        v
[IoT Devices/Sensors]
```

- **client_logger**: Collects and sends device/sensor data.
- **backend**: Axum-based web server, exposes APIs, handles alerts, nowcasts, and status.
- **wictk_core**: Shared business logic, data models, and integrations.

---

## Project Structure

```
.
├── backend/         # Main Axum web server (API, alerting, nowcasts)
│   └── src/handlers/
├── client_logger/   # Device-side logger and uploader
│   └── src/
├── wictk_core/      # Shared business logic, models, and integrations
│   └── src/
├── load_test/       # Locust load testing scripts
├── release/         # Kubernetes manifests and deployment configs
├── .github/         # CI/CD workflows and config
├── Dockerfile       # Container build
├── Makefile         # Build/test helpers
└── README.md        # (This file)
```

---

## Core Components

### backend/
- **Purpose**: Main API server (Axum), exposes endpoints for alerts, nowcasts, status, and location.
- **Key files**: `main.rs`, `handlers/`
- **Features**: Async, error handling, tracing, integrates with `wictk_core`.

### wictk_core/
- **Purpose**: Shared logic, data models, and integrations (e.g., MET, OpenWeatherMap).
- **Key files**: `alerts/`, `lightning/`, `locations/`, `nowcasts/`
- **Features**: Serde serialization, business rules, extensible modules.

### client_logger/
- **Purpose**: Collects sensor/device data and uploads to backend.
- **Key files**: `main.rs`, `device.rs`, `measurement.rs`
- **Features**: Local storage, batching, error recovery.

### load_test/
- **Purpose**: Load testing with Locust.
- **Key files**: `locustfile.py`, `requirements.txt`

### release/
- **Purpose**: Kubernetes deployment manifests.
- **Key files**: `deployment.yaml`, `service_loadbalancer.yaml`, `cronjob.yaml`

---

## Development Guidelines

- **Language**: Rust 2021 edition
- **Async**: Use `tokio` runtime, `async fn` for handlers
- **Error Handling**: Use `anyhow::Error` for main, `Result<T, E>` for fallible ops
- **Logging**: Use `tracing` crate, `#[instrument]` for tracing
- **Secrets**: Use `redact::Secret<T>` for sensitive data
- **JSON**: Use `serde` with `#[derive(Serialize, Deserialize)]`
- **Testing**: Use `#[cfg(test)]`, `pretty_assertions` for diffs
- **Imports**: Group by std/external/internal, alphabetical within groups
- **Naming**: `snake_case` for functions/vars, `PascalCase` for types

See [AGENTS.md](./AGENTS.md) for more details.

---

## Build, Test, and Run

**Build all crates:**
```sh
cargo build
# or
make build
```

**Run all tests:**
```sh
cargo test
# or
make test
```

**Check code:**
```sh
cargo check
# or
make check
```

**Run backend server:**
```sh
cd backend
cargo run
```

**Run client logger:**
```sh
cd client_logger
cargo run
```

**Load testing:**
```sh
cd load_test
pip install -r requirements.txt
locust
```

**Docker:**
```sh
docker build -t wictk .
docker-compose up
```

**Kubernetes:**
See `release/` for manifests.

---

## Contributing

- Fork the repo and create a feature branch
- Follow code style and guidelines above
- Add tests for new features
- Open a pull request with a clear description

---

## License

[MIT](./LICENSE) © 2025 Fredrik

---

## Contact

- **Author**: Fredrik
- **Issues**: [GitHub Issues](https://github.com/yourusername/wictk/issues)
- **Contributions**: Welcome! See [Contributing](#contributing)

---
