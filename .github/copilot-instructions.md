# GitHub Copilot Instructions for rust-stock

## Build, Run, and Lint

- **Build:** `cargo build` (dev) or `cargo build --release` (production)
- **Run:** `cargo run`
- **Lint:** `cargo clippy` (Ensure no warnings before committing)
- **Format:** `cargo fmt`
- **Test:** `cargo test` (Currently no tests, but use this to verify new tests)

## High-Level Architecture

- **Pattern:** Async TUI (Terminal User Interface) application.
- **Threading Model:**
  - **Main Thread:** Handles UI rendering (TUI) and input events (`crossterm`).
  - **Background Thread:** Polling loop that fetches stock data via HTTP.
  - **Communication:** Uses channels (`std::sync::mpsc`) to send API data from background thread to main thread.
- **Module Responsibilities:**
  - `src/main.rs`: Entry point, setup, and main event loop.
  - `src/app.rs`: State management and business logic.
  - `src/api.rs`: HTTP client wrapper for East Money API.
  - `src/model.rs`: Data structures (Stock, DTOs) and JSON deserialization.
  - `src/widget.rs`: UI rendering logic (layout, tables, charts).
  - `src/storage.rs`: Persists user configuration (stock list) to disk.

## Key Bindings (TUI)

- **General:**
  - `q`: Exit application
  - `r`: Refresh stock data immediately
- **Stock Management:**
  - `n`: Enter "Adding" mode to input a new stock code
  - `d`: Delete the currently selected stock
  - `u`: Move selected stock up (reorder)
  - `j`: Move selected stock down (reorder)
- **Navigation:**
  - `Up` / `Down`: Navigate the list
  - `Mouse Scroll`: Navigate the list
  - `Left Click`: Select a stock

## Key Conventions

- **State Management:**
  - The application uses a simple state machine (`AppState`) in `src/events.rs`:
    - `Normal`: Viewing list, standard navigation.
    - `Adding`: Input mode for new stock codes.
- **Dependency Management:**
  - Keep the binary size small.
  - Use `http_req` (blocking) instead of `reqwest` (heavy).
  - Use `tui` and `crossterm` for UI.
- **East Money API Quirks:**
  - **Data Mapping:** Uses `serde` with `rename` attributes in `src/model.rs` (`RawStock` struct) to map fields like `f12` to `code`.
  - **Data Scaling:** Raw API values are integers.
    - **CN (A-Share):** Divide price/values by **100**.
    - **HK/US/UK:** Divide price/values by **1000** (Markets 116, 105-107, 155).
    - **Percentages:** Generally divide by **100**.
  - **SecIDs:** Format is `market_id.code` (e.g., `1.600519` for SH, `0.000001` for SZ, `105.NVDA` for US).
- **UI & UX:**
  - Ensure TUI remains responsive; blocking I/O must happen in the background thread.
  - Handle Unicode width for Chinese characters correctly (use `unicode-width` crate).
