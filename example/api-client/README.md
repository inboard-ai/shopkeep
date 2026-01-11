# api-client

An HTTP client example demonstrating how to interact with the shopkeep extension registry API.

## Running

Run from the repository root (`shopkeep/`). The repo includes a sample registry at `./registry` with two extensions (`kv` and `polygon`) for testing.

**Terminal 1** - Start the shopkeep server:

```bash
cargo run -- --registry-path ./registry
```

The server starts on `http://localhost:8080` by default.

**Terminal 2** - Run the example client:

```bash
cargo run -p api-client
```

Or with a custom URL:

```bash
cargo run -p api-client -- --url http://localhost:3000
```

## What it does

The example demonstrates all the main API endpoints:

1. `GET /api/v1/extensions` - List all extensions with pagination
2. `GET /api/v1/extensions/{id}` - Get detailed info for an extension
3. `GET /api/v1/extensions/{id}/versions` - List all versions
4. `GET /api/v1/extensions/{id}/latest/download` - Download the latest package

## Example output

```
Shopkeep API Client Demo
========================

Connecting to: http://localhost:8080

1. Listing extensions...

   Found 2 extension(s) (page 1/1):

   - kv v0.1.0
     A resource-based key-value store with instance-specific state.
     Author: Andy Terra, License: MIT

   - polygon v0.1.0
     Draw polygons with configurable colors and fill.
     Author: Andy Terra, License: MIT

2. Getting details for 'kv'...

   Name: Key-Value Store
   Version: 0.1.0
   ...

3. Listing versions for 'kv'...

   - v0.1.0
     Size: 247000 bytes
     SHA256: 2d4da317496f5d0246838492cb524473faa6ea43ac7adac350c778a8a97b3c4e
     ...

4. Downloading latest version of 'kv'...

   Downloaded 247000 bytes
   Content-Disposition: attachment; filename="kv-0.1.0.empkg"

Demo complete!
```
