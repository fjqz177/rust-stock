# GitHub Copilot Instructions for rust-stock

## Build, Run, and Lint

- **Build:** `cargo build` (dev) or `cargo build --release` (production)
- **Run:** `cargo run`
- **Lint:** `cargo clippy` (Ensure no warnings before committing)
- **Format:** `cargo fmt`
- **Test:** `cargo test`
- **Single test:** `cargo test --lib api::tests::to_secid_numeric_blind` (example)

## High-Level Architecture

- **Pattern:** Async TUI (Terminal User Interface) application.
- **Threading Model:**
  - **Main Thread:** Handles UI rendering (TUI) and input events (`crossterm`).
  - **Background Thread:** Polling loop that fetches stock data via HTTP.
  - **Communication:** Uses channels (`std::sync::mpsc`) to send API data from background thread to main thread.
  - **Event Flow:** `main.rs` draws/polls, `events.rs` mutates `App` and calls `App::on_tick`, which drains channel updates and triggers periodic refreshes in `Normal` state.
- **Module Responsibilities:**
  - `src/main.rs`: Entry point, setup, and main event loop.
  - `src/app.rs`: State management and business logic.
  - `src/events.rs`: Keyboard/mouse handling and tick dispatch to the app.
  - `src/api.rs`: HTTP client wrapper for East Money API.
  - `src/model.rs`: Data structures (Stock, DTOs) and JSON deserialization.
  - `src/widget.rs`: UI rendering logic (layout, tables, charts).
  - `src/storage.rs`: Persists user configuration (stock list) to disk.
  - `src/lib.rs`: Module exports and shared type aliases.

## Key Bindings (TUI)

- **General:**
  - `q`: Exit application
  - `r`: Refresh stock data immediately
- **Stock Management:**
  - `n`: Enter "Adding" mode to input a new stock code
  - `d`: Delete the currently selected stock
  - `u`: Move selected stock up (reorder)
  - `j`: Move selected stock down (reorder)
- **Adding Mode:**
  - `Enter`: Confirm and add input
  - `Esc`: Cancel input
- **Navigation:**
  - `Up` / `Down`: Navigate the list
  - `Mouse Scroll`: Navigate the list
  - `Left Click`: Select a stock

## Key Conventions

- **State Management:**
  - The application uses a simple state machine (`AppState`) in `src/events.rs`:
    - `Normal`: Viewing list, standard navigation.
    - `Adding`: Input mode for new stock codes.
- **Stock Code Normalization:**
  - Matching uses `normalize_code_for_match` (strip leading `x`, remove numeric market prefix before `.`, uppercase) to keep user input stable.
  - Non-numeric prefixes like `RR.` keep the dot as part of the code (UK tickers).
  - Manual secid passthrough uses `x` prefix (e.g., `x105.NVDA`).
  - Blind numeric codes map to `1.` (SH), `0.` (SZ), `116.` (HK); alpha codes map to `105/106/107.` (US) and `155.` (UK).
- **User Code Preservation:**
  - `App::update_stocks` replaces fields with API data but restores the original user-entered `code` to preserve manual prefixes.
- **Refresh Throttling:**
  - `App::refresh_stocks` uses a `sync_channel` of size 1 and drops refresh requests when full.
- **Dependency Management:**
  - Keep the binary size small.
  - Use `http_req` (blocking) instead of `reqwest` (heavy).
  - Use `tui` and `crossterm` for UI.
- **Storage Location:**
  - Default JSON storage is `~/.stocks.json`, override with `RUST_STOCK_DB_PATH`.
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
