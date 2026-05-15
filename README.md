# VelocityRL

A powerful, user-friendly native Rust engine and Tauri desktop app for performing visual asset swaps in Rocket League.

## Overview

**VelocityRL** provides a premium desktop interface that allows you to swap in-game items (e.g., swapping a standard boost for Alpha Reward). It uses a 100% native Rust engine to parse UPK files, handle AES decryption, LZO/ZLIB decompression, and perform structural structural re-alignment via the "Dummy Pivot" technique.

## Features

- **100% Native Rust Engine**: High-performance UPK manipulation without Python dependencies.
- **Dummy Pivot Architecture**: Safe asset swapping that preserves package integrity and prevents game crashes.
- **Tauri Desktop UI**: A sleek, modern interface for managing your item collection and performing swaps.
- **Psynet API Server**: A standalone Axum-powered web server for hosting the item database.
- **Automated Backups**: Automatically backs up original game assets before patching, with easy restore features.

## API Hosting

To run the standalone API server (for `api.velocityrl.me`):

```bash
cd src-tauri
cargo run --bin velocity-api
```

The server runs on port `3000` by default.

## Installation

1. Clone the repository.
2. Install Rust and Node.js.
3. Run `npm install` and `npm run dev` to start the app.

## Development

- `src-tauri/src/engine`: The core Rust engine.
- `src-tauri/src/api_main.rs`: Standalone API server.
- `ui/`: The React-based frontend.
