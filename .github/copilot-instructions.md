<!-- .github/copilot-instructions.md for doc-simfinder -->
# doc-simfinder — Copilot instructions

Short: concurrent document similarity search (Rust). Help contributors by following the file-level patterns, public APIs, and conventions below.

**Big Picture**
- **Purpose**: Index and score documents by similarity to a query using sliding-window chunking + fuzzy matching (see `src/analysis.rs`).
- **Major components**: `src/file_walker.rs` (find files), `src/config.rs` (Config struct and defaults), `src/analysis.rs` (chunking and scoring, uses `rayon`), `src/cli.rs` (planned CLI), `src/main.rs` (example usage). Public modules are exported in `src/lib.rs`.

**Key implementation patterns**
- **Chunking & sliding window**: `analysis.rs` builds overlapping `Chunk`s from full file text (uses `fs::read_to_string`). Keep this pattern when changing scoring logic; functions like `get_chunks`, `calculate_sliding_window` and `score_chunk` are central.
- **Similarity normalization**: Scores are normalized against an approximate optimal score (`calculate_approximate_optimal_score`) before thresholding and selecting top N chunks (`top_n` from `Config`).
- **Parallel processing**: Use `rayon`'s `par_iter()` where present (scoring uses `par_iter()` with `.with_min_len(50)`). Preserve thread-safety and avoid global mutability.
- **Config-driven behavior**: `Config` (in `src/config.rs`) holds search parameters and defaults via `Default`. Many modules expect a `&Config` reference; prefer extending `Config` rather than scattering globals.

**Dependencies & integration points**
- See `Cargo.toml`: `clap`, `fuzzy-matcher`, `globset`, `inquire`, `rayon`, `walkdir`, `thiserror`.
- `analysis::score_file(path, &config)` is a primary entrypoint for computing similarity for a single file; `file_walker::walk_from_root(&config)` returns the input file set. Keep these function signatures stable when the CLI or other callers are added.

**Developer workflows (what works now)**
- Build: `cargo build` (from repo root).
- Run the example main: `cargo run` (the current `src/main.rs` constructs a manual `Config` to exercise `walk_from_root` and `score_file`).
- Tests: none present; use `cargo test` after adding tests.

**Project-specific guidance for edits**
- If you change chunking/IO, note that `analysis.rs` currently reads full files into memory. For large-file support prefer streaming/BufReader and update `get_chunks` accordingly.
- When adding CLI flags, implement in `src/cli.rs` using `clap` (already listed in `Cargo.toml`) and map flags into `Config`. `src/main.rs` contains a minimal manual example to mirror.
- Error types live in `src/errors.rs` and favor `thiserror` style; add variants there instead of ad-hoc strings.
- Keep public surface in `src/lib.rs` in sync with module exports when adding features.

**Examples (copyable patterns found in the repo)**
- Score a file from `main`: `let res = score_file(res.files[2].as_path(), &config);`
- Use defaults: `let cfg = Config::default();` — add or override fields as needed.

**What to watch for / current TODOs**
- `src/cli.rs` is empty — implement CLI there.
- `src/config.rs::validate` is `todo!()` — if you add CLI argument parsing, implement validation logic here.
- `analysis.rs` prints debug info with `println!` in several functions (`optimal score`, fuzzy match results); prefer `tracing` or conditional debug logging in PRs.

If anything above is unclear or you want more detail on a particular file, tell me which file or flow you want expanded and I will iterate the instructions.
