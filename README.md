<div align="center">

# shopkeep

[![Crates.io](https://img.shields.io/crates/v/shopkeep.svg)](https://crates.io/crates/shopkeep)
[![Documentation](https://docs.rs/shopkeep/badge.svg)](https://docs.rs/shopkeep)
[![License](https://img.shields.io/crates/l/shopkeep.svg)](https://github.com/inboard-ai/shopkeep/blob/main/LICENSE)

HTTP server for the [emporium](https://github.com/inboard-ai/emporium) extension marketplace

</div>

## Quick Start

```bash
cargo install shopkeep
shopkeep --registry-path ./extensions
```

The server starts on `http://0.0.0.0:8080` by default.

## Configuration

Configuration can be provided via file, environment variables, or CLI arguments (in order of precedence).

**CLI arguments:**
```bash
shopkeep --bind 127.0.0.1 --port 3000 --registry-path /path/to/extensions
```

**Environment variables:**
```bash
export SHOPKEEP_BIND=127.0.0.1
export SHOPKEEP_PORT=3000
export SHOPKEEP_REGISTRY_PATH=/path/to/extensions
```

**Config file (`shopkeep.toml`):**
```toml
bind = "127.0.0.1"
port = 3000

[registry]
type = "filesystem"
path = "/path/to/extensions"
```

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Health check |
| `GET` | `/api/v1/extensions` | List extensions (supports `?q=`, `?category=`, `?page=`, `?per_page=`) |
| `GET` | `/api/v1/extensions/{id}` | Get extension details |
| `GET` | `/api/v1/extensions/{id}/versions` | List available versions |
| `GET` | `/api/v1/extensions/{id}/versions/{version}` | Get version metadata |
| `GET` | `/api/v1/extensions/{id}/versions/{version}/download` | Download extension package |
| `GET` | `/api/v1/extensions/{id}/latest/download` | Download latest version |

## Registry Structure

Extensions are stored as `.tar.gz` packages with the following structure:

```
registry/
├── my-extension/
│   ├── 0.1.0.tar.gz
│   ├── 0.2.0.tar.gz
│   └── manifest.toml
└── another-extension/
    ├── 1.0.0.tar.gz
    └── manifest.toml
```

## License

MIT
