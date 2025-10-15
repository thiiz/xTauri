# Technology Stack

## Frontend

- **Framework**: React 18.3+ with TypeScript 5.6+
- **Build Tool**: Vite 6.0+
- **State Management**: Zustand 5.0+
- **Virtual Scrolling**: react-virtuoso 4.14+
- **Video Streaming**: hls.js 1.6+

## Backend

- **Runtime**: Tauri 2.x (Rust + WebView)
- **Database**: SQLite via rusqlite 0.30+
- **HTTP Client**: reqwest 0.12+ (Rust)
- **Async Runtime**: tokio 1.x

## Package Manager

Use **bun** for all JavaScript/TypeScript operations. Do not use npm or yarn.

## Common Commands

```bash
# Development
bun dev:tauri          # Run app in development mode
bun dev                # Run Vite dev server only

# Building
bun build:tauri        # Build production app
bun build              # Build frontend only

# Code Quality
bun type-check         # TypeScript type checking
bun lint               # Run linter
bun format             # Format code with Prettier
bun format:check       # Check formatting

# Maintenance
bun clean              # Remove dist and target folders
bun clean:full         # Full clean including node_modules
bun install:all        # Install all dependencies (JS + Rust)

# Tauri
bun tauri dev          # Alternative dev command
bun tauri build        # Alternative build command
bun check:tauri        # Check Tauri setup
```

## TypeScript Configuration

- **Target**: ES2020
- **Module**: ESNext with bundler resolution
- **Strict Mode**: Enabled
- **JSX**: react-jsx
- Unused locals and parameters are errors
- No fallthrough cases in switch statements

## Build Configuration

- **Dev Server Port**: 1420 (strict)
- **HMR Port**: 1421
- Vite ignores `src-tauri` directory
- Clear screen disabled for Rust error visibility

## Key Dependencies

### Frontend
- `@tauri-apps/api` - Tauri JavaScript API
- `@tauri-apps/plugin-opener` - Open external links
- `hls.js` - HLS video streaming
- `zustand` - State management

### Backend (Rust)
- `rusqlite` - SQLite database (bundled)
- `reqwest` - HTTP client (rustls-tls)
- `serde` / `serde_json` - Serialization
- `tokio` - Async runtime
- `dashmap` - Concurrent caching
- `keyring` - Secure credential storage
- `aes` / `base64` - Encryption for Xtream API

## Testing & Benchmarking

- `tokio-test` - Async testing
- `criterion` - Benchmarking with HTML reports
- `mockall` - Mocking
- `rstest` - Parameterized tests
- `wiremock` - HTTP mocking

Benchmarks available for: fuzzy search, database operations, M3U parsing, and caching.
