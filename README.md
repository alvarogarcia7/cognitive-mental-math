# Cognitive Mental Math

A Rust-based GUI application to exercise and improve mental math abilities through interactive quizzes and spaced repetition learning.

![CI Status](https://github.com/alvarogarcia7/cognitive-mental-math/workflows/CI/badge.svg)

## Features

- **Interactive Quiz Interface** - GUI-based mental math quizzes with real-time feedback
- **Spaced Repetition** - SM2 (SuperMemo 2) algorithm implementation for optimized learning
- **Performance Tracking** - Detailed statistics on accuracy, timing, and learning progress
- **Persistent Storage** - SQLite database for maintaining quiz history and performance data
- **Multiple Operations** - Support for various mathematical operations (addition, multiplication)
- **Deck Management** - Organize questions into decks and track completion
- **Comprehensive Testing** - 164 unit, integration, and end-to-end tests with full CI/CD pipeline

## Prerequisites

- Rust 1.70 or later (2024 edition)
- Cargo
- System dependencies:
  - GTK 3 (for GUI rendering)
  - X11 and XCB libraries (for window management)
  - SQLite development libraries

### Installing System Dependencies

**Ubuntu/Debian:**
```bash
sudo apt-get install libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libsqlite3-dev
```

**macOS:**
```bash
brew install gtk+3 sqlite3
```

## Installation

1. Clone the repository:
```bash
git clone https://github.com/alvarogarcia7/cognitive-mental-math.git
cd cognitive-mental-math
```

2. Build the project:
```bash
cargo build --release
```

3. (Optional) Install git hooks for development:
```bash
./install-hooks.sh
```

## Usage

### Running the Application

To run the application:
```bash
make run
```

### Development Mode

Run with a test database (in-memory or custom path):
```bash
# In-memory database
make run-dev-memory

# Custom database file
make run-dev
```

### Building without Running

```bash
cargo build
cargo build --release
```

## Testing

### Run All Tests
```bash
make test
```

### Run Specific Test Suite
```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test integration_tests

# End-to-end tests
cargo test --test e2e_tests
```

### Run Tests with Output
```bash
cargo test -- --nocapture
```

### Using Make Commands
```bash
make test              # Run all tests
make build_and_test    # Build and run tests
make pre-commit        # Format, build, test, and lint
make pre-push          # Run comprehensive checks
```

## Architecture

### Core Components

**GUI Module** (`src/gui.rs`)
- Built with `eframe` (egui framework)
- Handles question display, answer input, and results visualization
- Manages application state and user interactions

**Quiz Service** (`src/quiz_service.rs`)
- Manages question flow and delivery
- Handles answer submission and validation
- Tracks results and timing data

**Database Layer** (`src/database.rs`)
- SQLite-based persistence
- Manages operations, answers, decks, and review items
- Computes comprehensive statistics (accuracy, timing, streaks)

**Spaced Repetition** (`src/spaced_repetition.rs`)
- SM2 (SuperMemo 2) algorithm implementation
- Review scheduling and ease factor management
- Optimizes learning efficiency

**Operations Module** (`src/operations.rs`)
- Generates mathematical questions
- Validates user answers
- Supports multiple operation types

## Debugging

### Enable Logging

Set the `RUST_LOG` environment variable:
```bash
RUST_LOG=debug cargo run
RUST_LOG=memory_practice=trace cargo run
```

### Database Inspection

The application uses SQLite, which stores data in:
- Default: `~/.local/share/cognitive-mental-math/memory_practice.db`
- Test mode: In-memory or custom path

You can inspect the database with:
```bash
sqlite3 ~/.local/share/cognitive-mental-math/memory_practice.db
```

## CI/CD Pipeline

The project uses GitHub Actions for continuous integration. See `.github/workflows/ci.yml`

## License

[License information to be added]

## Development History

See `PROMPTS.md` for detailed development history and notes.

---

**Last Updated:** 2025-11-12
