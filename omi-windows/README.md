# Omi Windows

Windows desktop app for Omi — always-on AI companion. Built with Dioxus (Rust) + Backend-Rust sidecar.

## Prerequisites

- [Rust toolchain](https://rustup.rs/) (1.80+)
- [Node.js](https://nodejs.org/) 18+ (for agent runtime)
- [Docker](https://www.docker.com/) (for self-hosted Python backend)

## Quick Start

```powershell
cd omi-windows
cargo run -p omi-app
```

This launches the Dioxus desktop app and attempts to connect to the Backend-Rust sidecar on `localhost:10201`.

## Running the Backend Sidecar

In a separate terminal:

```powershell
cd desktop\Backend-Rust
copy .env.example .env
# Fill in API keys in .env
cargo run
```

## Project Structure

```
crates/
  omi-app/            Dioxus desktop UI (main binary)
  omi-capture/        DXGI screen capture + Windows OCR
  omi-audio/          Mic + system audio via WASAPI/cpal
  omi-ble/            BLE wearable support via btleplug (optional, see below)
  omi-db/             Local SQLite storage
  omi-transcription/  Deepgram WebSocket client
```

## BLE Wearable Support

BLE support is **disabled by default** due to hardware limitations — it requires a BLE-capable
adapter and an Omi wearable device for testing/development.

To build with BLE enabled:

```powershell
cargo run -p omi-app --features ble
```

## License

MIT
