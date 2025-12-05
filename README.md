# doc-simfinder

A fast, concurrent document similarity search tool written in Rust. Search through your text documents using fuzzy matching with sliding-window chunking to find the most relevant files and text snippets.

## Features

- **Parallel processing** - Uses Rayon for concurrent file analysis
- **Sliding-window chunking** - Scores overlapping text segments for better accuracy
- **Two modes** - CLI for quick searches, TUI for interactive exploration
- **Real-time feedback** - TUI mode shows found files as you type paths
- **Syntax highlighting** - Color-coded results with matched text highlighted

## Setup

### Prerequisites
- Rust toolchain (1.70+)
- Cargo package manager

### Installation
```bash
git clone https://github.com/guilloboi1917/doc-simfinder.git
cd doc-simfinder
cargo build --release
```

The compiled binary will be at `target/release/doc-simfinder` (or `doc-simfinder.exe` on Windows).

## Usage

### CLI Mode
Quick search from command line:
```bash
# Basic search
doc-simfinder --query "your search text" --search-path ./documents

# With custom options
doc-simfinder --query "rust async" --search-path ./src --top-n 10 --threshold 0.3
```

**CLI Options:**
- `--query, -q` - Search query text (required)
- `--search-path, -p` - Directory to search (default: current directory)
- `--top-n, -n` - Number of top results per file (default: 3)
- `--threshold, -t` - Minimum similarity score (default: 0.4)
- `--window-size, -w` - Sliding window size (default: 150)

### TUI Mode
Interactive terminal interface:
```bash
doc-simfinder --tui
```

**TUI Controls:**
- `Tab` / `Shift+Tab` - Navigate between fields
- Type to edit path and query inputs
- `Enter` - Start analysis (when ready)
- `↑/↓` or `j/k` - Navigate results
- `Enter` - View file details
- `Backspace` - Go back
- `Ctrl+R` - Reanalyze
- `Ctrl+Q` or `Ctrl+C` - Quit

## Limitations

- **File types**: Currently supports `.txt` and `.md` files only
- **Text input**: No cursor movement in TUI - use backspace to edit from the end
- **No filtering**: Results cannot be filtered by filename or path patterns yet
- **Binary files**: Automatically skipped but may cause warnings for edge cases

## Configuration

Default settings can be found in `src/config/mod.rs`. Key parameters:
- Window size: 150 characters
- Max window size: 500 characters  
- Similarity threshold: 0.4
- Top N chunks per file: 3
- Thread count: Auto-detected based on CPU cores

## Examples

Search for Rust error handling patterns:
```bash
doc-simfinder --query "Result<T, E>" --search-path ./rust-projects --top-n 5
```

Find documentation about configuration:
```bash
doc-simfinder --tui
# Then type path: ./docs
# Then type query: configuration settings
# Press Enter to analyze
```

## License

See [LICENSE](LICENSE) file for details.