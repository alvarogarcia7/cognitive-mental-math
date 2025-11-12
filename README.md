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
cargo run --release
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

The project includes 164 comprehensive tests:

### Run All Tests
```bash
cargo test
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

## Project Structure

```
cognitive-mental-math/
├── src/
│   ├── main.rs                    # Application entry point
│   ├── gui.rs                     # GUI implementation (egui)
│   ├── quiz_service.rs            # Quiz logic and flow
│   ├── database.rs                # Database operations
│   ├── database_factory.rs        # Database configuration
│   ├── operations.rs              # Math operations handling
│   ├── spaced_repetition.rs       # SM2 algorithm implementation
│   ├── answer_evaluator_service.rs # Answer evaluation logic
│   ├── deck.rs                    # Deck management
│   ├── row_factories.rs           # Database row mapping
│   ├── time_format.rs             # Time formatting utilities
│   ├── lib.rs                     # Public module exports
│   └── bin/
│       ├── performance_stats.rs   # Performance analysis tool
│       └── sm2_scheduler.rs       # Spaced repetition demo
├── tests/
│   ├── integration_tests.rs       # Integration tests
│   └── e2e_tests.rs               # End-to-end tests
├── migrations/
│   └── V1__initial_schema.sql    # Database schema
├── .github/
│   └── workflows/
│       └── ci.yml                 # CI/CD pipeline
├── hooks/
│   ├── pre-commit                 # Pre-commit hook
│   └── pre-push                   # Pre-push hook
├── Cargo.toml                     # Project manifest
├── Makefile                       # Build automation
└── backlog/                       # Feature planning documents
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

### Design Patterns

- **Template Method Pattern** - Used in statistics queries for consistent computation across different metrics
- **Factory Pattern** - Database factory for flexible configuration (memory, test, production modes)
- **Service Layer Pattern** - Quiz and answer evaluation services encapsulate business logic

## Development Workflow

### Code Quality Standards

1. **Format Code:**
```bash
cargo fmt
```

2. **Run Linter:**
```bash
cargo clippy
```

3. **Check Everything:**
```bash
make check    # Runs format check, clippy, and tests
```

4. **Pre-Commit Checklist:**
```bash
make pre-commit    # Format, build, test, and lint
```

### Git Workflow

1. Create a feature branch:
```bash
git checkout -b feature/issue-XXX-description
```

2. Make changes and commit:
```bash
git add .
git commit -m "Fix #XXX: Descriptive message"
```

3. Push and create a pull request:
```bash
git push origin feature/issue-XXX-description
```

### Commit Message Format

Follow conventional commits format:
- `fix: Issue description` - Bug fixes
- `feat: Feature description` - New features
- `refactor: Changes description` - Code refactoring
- `test: Test description` - Test additions/improvements
- `docs: Documentation description` - Documentation changes
- `chore: Maintenance description` - Maintenance tasks

Include issue numbers when applicable: `fix #123: Clear issue description`

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

## Dependencies

Key dependencies:
- **eframe 0.29** - GUI framework (egui)
- **rusqlite 0.32** - SQLite database bindings
- **refinery 0.8** - Database migrations
- **rand 0.8** - Random number generation
- **chrono 0.4** - Date and time handling
- **sra 0.1** - Spaced repetition algorithms
- **log 0.4 & env_logger 0.11** - Logging framework

See `Cargo.toml` for complete dependency list.

## CI/CD Pipeline

The project uses GitHub Actions for continuous integration:
- Builds on every push
- Runs 164 comprehensive tests
- Checks code formatting
- Runs clippy linter
- Supports multiple platforms

See `.github/workflows/ci.yml` for pipeline configuration.

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch for your changes
3. Ensure all tests pass: `cargo test`
4. Follow code style: `cargo fmt`
5. Address linting issues: `cargo clippy`
6. Write clear commit messages
7. Submit a pull request with a detailed description

## Backlog & Future Enhancements

The project has planned features in the `backlog/` directory:

- **Repeating Failed Questions** - Focus on questions with lower accuracy
- **Difficulty Levels** - Adjust question difficulty based on performance
- **Custom Deck Creation** - Allow users to create personalized question decks
- **Advanced Spaced Repetition** - Enhancements to the SM2 algorithm
- **Analytics Dashboard** - Visual performance metrics and progress tracking

## License

[License information to be added]

## Support

For issues, questions, or suggestions, please open an issue on GitHub or refer to the documentation in the `.github/` and `hooks/` directories.

## Development History

See `PROMPTS.md` for detailed development history and notes.

---

**Last Updated:** 2025-11-12
