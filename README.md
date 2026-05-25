# ValidResult Frontend

Frontend for the ValidResult AI evaluation platform, built with **Leptos 0.8** (CSR) and **Tailwind CSS**, compiled to WASM.

## Project Structure

```
src/
  main.rs      ← App entry point
  table.rs     ← Results table component
index.html     ← Entry point
input.css      ← Tailwind entry
Trunk.toml
Cargo.toml
```

## Setup

### Prerequisites

```bash
cargo install trunk
rustup target add wasm32-unknown-unknown
```

### Run in development

```bash
trunk serve --open
```

### Build for production

```bash
trunk build --release
# Output in ./dist/
```

Then serve the `dist/` folder with any static file host (Nginx, Caddy, GitHub Pages, etc.).
