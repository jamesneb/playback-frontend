# Playback Frontend

A SvelteKit frontend application with WebAssembly support for playback telemetry visualization.

## Features

- 🚀 SvelteKit with TypeScript
- 🔧 WebAssembly support via Vite plugins
- 🌍 Environment-based configuration (local/dev/staging/prod)
- 📦 Modern build system with Vite

## Development

### Prerequisites

- Node.js (v18+ recommended)
- npm

### Installation

```bash
npm install
```

### Running the Application

For local development:
```bash
npm run dev
# or
npm run dev:local
```

For different environments:
```bash
npm run dev:dev      # Development environment
```

### Building

```bash
npm run build          # Default build
npm run build:local    # Local environment
npm run build:dev      # Development environment
npm run build:staging  # Staging environment
npm run build:prod     # Production environment
```

### Type Checking

```bash
npm run check          # Run once
npm run check:watch    # Watch mode
```

### Linting

```bash
npm run lint
```

## Project Structure

```
src/
├── lib/
│   ├── config.ts      # Environment configuration
│   ├── types.ts       # TypeScript type definitions
│   └── wasm.ts        # WebAssembly loader utility
├── routes/
│   ├── +layout.svelte # Root layout
│   └── +page.svelte   # Homepage
├── app.d.ts           # App type definitions
└── app.html           # HTML template

environments/
├── local/             # Local development
├── dev/               # Development server
├── staging/           # Staging server
└── prod/              # Production server
```

## Environment Configuration

Each environment has its own `.env` file in the `environments/` directory. The application automatically loads the correct configuration based on the build/dev command used.

Environment variables are prefixed with `VITE_` to be accessible in the frontend.

## WebAssembly Support

The project includes WASM support through Vite plugins:
- `vite-plugin-wasm` - Enables WASM module loading
- `vite-plugin-top-level-await` - Allows top-level await for WASM initialization

Use the `WasmLoader` utility class in `src/lib/wasm.ts` to load and manage WASM modules.