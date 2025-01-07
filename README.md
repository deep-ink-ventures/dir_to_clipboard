<img src="https://rustacean.net/assets/rustacean-flat-happy.svg" width="100" height="100" align="right" />

# dir_to_clipboard

A simple command-line utility that copies directory contents and file contents to your clipboard. Bring your GPT up to speed with your codebase fast.

## Features

- 📁 Copy directory listings (`ls -l` output)
- 📄 Copy file contents
- 🌳 Optional recursive directory traversal
- 🔍 File pattern filtering (e.g., `*.rs`, `*.txt`)
- 📋 Direct clipboard integration
- 🚀 Fast and memory efficient
- ✅ Base directory selection with `--base-dir`
- 🔓 `.gitignore` support for ignoring files (toggleable with `--no-ignore`)

## Installation

### Prerequisites

- Rust and Cargo (if building from source)
- A clipboard-compatible system (macOS, Linux with X11)
- **Linux users**: Install `xsel` for clipboard functionality:
  ```bash
  sudo pacman -S xsel  # Arch Linux
  sudo apt-get install xsel  # Debian/Ubuntu
  sudo yum install xsel  # Red Hat/CentOS
  ```

### Building from Source

```bash
# Clone the repository
git clone https://github.com/deep-ink-ventures/dir_to_clipboard
cd dir_to_clipboard

# Build for release
cargo build --release

# Install globally (macOS/Linux)
sudo cp target/release/dir_to_clipboard /usr/local/bin/

# Or install for current user only
mkdir -p ~/.local/bin
cp target/release/dir_to_clipboard ~/.local/bin/
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc  # or ~/.bashrc
```

## Usage

### Basic Usage

```bash
# Copy current directory contents
dir_to_clipboard

# Copy recursively
dir_to_clipboard --recursive

# Filter specific files
dir_to_clipboard --filter "*.rs"

# Combine options
dir_to_clipboard --recursive --filter "*.rs"

# Set base directory
dir_to_clipboard --base-dir /path/to/dir

# Disable .gitignore filtering
dir_to_clipboard --no-ignore

# Short form
dir_to_clipboard -r -f "*.rs" -d /path/to/dir --no-ignore
```

### Output Format

The copied content will be formatted as follows:

```
=== Directory: ./path/to/dir ===
[ls -l output here]

=== File: ./path/to/file.rs ===
[file contents here]
```

### Command-line Options

| Option              | Short | Description                                      |
|---------------------|-------|--------------------------------------------------|
| `--base-dir <DIR>`  | `-d`  | Set the base directory (default: current dir)   |
| `--recursive`       | `-r`  | Recursively process subdirectories              |
| `--filter <PATTERN>`| `-f`  | Filter files by pattern (e.g., "*.rs")          |
| `--no-ignore`       |       | Disable `.gitignore` filtering                  |
| `--help`            | `-h`  | Show help message                               |
| `--version`         | `-V`  | Show version information                        |

## Smart Directory Handling

When using recursive mode with a filter:
- Only directories containing matching files are included
- Empty directories are automatically skipped
- Directory listings are shown before their file contents

## Dependencies

- `clipboard` - System clipboard integration
- `walkdir` - Directory traversal
- `anyhow` - Error handling
- `clap` - Command-line argument parsing
- `glob` - File pattern matching
- `gitignore` - `.gitignore` file handling
- `xsel` (Linux) - Clipboard management
- `tempfile` - For tests

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- The Rust community for excellent crates
- Ferris the crab for being an awesome mascot 🦀;

