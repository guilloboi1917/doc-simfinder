> **Note:** This README was partially AI-generated to document the implemented solution.

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

The compiled binary will be at `target/release/doc-simfinder`

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
- `--file-exts` - File extensions, comma-delimited (default: .txt, .md)


### TUI Mode
Interactive terminal interface:
```bash
doc-simfinder --tui
```

**TUI Controls:**
- `Ctrl+j` / `Ctrl+k` - Navigate between fields
- Type to edit path and query inputs
- `Enter` - Start analysis (when ready)
- `↑/↓` or `j/k` - Navigate results
- `Enter` - View file details
- `Backspace` - Go back
- `Ctrl+R` - Reanalyze
- `Ctrl+O` - Open file path location (When viewing results)
- `Ctrl+Q` or `Ctrl+C` - Quit

## Limitations

- **File types**: Supports common utf-8 files such as `.txt`, `.md`, and only `.pdf` binary files (PDF text extraction via lopdf)
- **Text input**: No cursor movement in TUI - use backspace to edit from the end
- **No filtering**: Results cannot be filtered by filename or path patterns yet
- **PDF limitations**: Image-only/scanned PDFs cannot be processed; 10MB size limit for memory safety

## Configuration

Default settings can be found in `src/config/mod.rs`. Key parameters:
- Window size: 500 characters
- Max window size: 5000 characters  
- Similarity threshold: 0.75
- Top N chunks per file: 5
- Thread count: Auto-detected based on CPU cores

## Examples

Search for Rust error handling patterns:
```bash
doc-simfinder --query "Result" --search-path ./rust-projects --top-n 5
```

## License

See [LICENSE](LICENSE) file for details.