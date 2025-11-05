# PROMPTS

## 2025-11-05 14:57:14 AGB - Original Prompt

Create a rust program that has a GUI. The goal is to exercise the memory of the user by performing basic mathematical operations. Separate the responsibilities into modules.

The program should:
- Ask the user for multiplication and addition operations
- Use between one and two digits per operand
- Only one operand for now (note: interpreted as one operation type at a time)
- Send a block of 10 questions at once
- Store each operation into a SQLite database
- Store the correct (or wrong) answer from the user into another table
- Time the amount spent thinking

## 2025-11-05 15:20:13 AGB - Testing Implementation

Create unit, integration, and end to end tests for this application.
Make the necessary splitting into functions and modules so it is possible to test.
Ensure all tests are passing.
Create a new entry in PROMPTS with the timestamp with my prompt.

### Result

Implemented comprehensive testing suite with all tests passing:

**Unit Tests (21 tests)**
- operations.rs: 12 tests covering Operation creation, answer checking, string formatting, type methods, random generation, and question block generation
- database.rs: 9 tests covering database creation, operation insertion, answer insertion, data retrieval, and relationship integrity

**Integration Tests (6 tests)**
- Full workflow from operation creation to database storage
- Question block storage verification
- Correct and incorrect answer handling
- Multiple operations with answers
- Operation type persistence in database

**End-to-End Tests (11 tests)**
- Complete application flow testing
- Answer submission workflow
- State transitions (ShowingQuestions → ShowingResults)
- Input validation (empty answers, invalid answers)
- Results tracking and correctness verification
- Database persistence across submissions
- Timing data recording

**Refactoring for Testability:**
1. Added `lib.rs` to expose modules for integration and E2E tests
2. Database module: Added query methods (`get_operation`, `get_answer`, `count_operations`, `count_answers`) and record structs (`OperationRecord`, `AnswerRecord`)
3. GUI module: Made `QuestionResult` and `AppState` public, added helper methods for testing (`get_current_state`, `get_current_question_index`, `get_results`, `set_answer`, `submit_answer`)

**Test Results:**
```
running 21 tests (unit tests) - all passed ✓
running 11 tests (e2e tests) - all passed ✓
running 6 tests (integration tests) - all passed ✓

Total: 38 tests passed, 0 failed
```

## 2025-11-05 15:28:22 AGB - Git Hooks Implementation

Create git hooks: on pre-commit, compile and run tests, on pre-push, do nothing.

### Result

Successfully implemented git hooks for the project:

**Pre-commit Hook** (`.git/hooks/pre-commit`)
- Automatically runs `cargo test --quiet` before each commit
- If tests fail, the commit is aborted with an error message
- If tests pass, the commit proceeds normally
- Provides clear feedback with ✅ or ❌ indicators

**Pre-push Hook** (`.git/hooks/pre-push`)
- Exists but performs no actions (as requested)
- Simply exits with success status

**Implementation Details:**
1. Created executable bash scripts in `.git/hooks/`
2. Made both hooks executable with `chmod +x`
3. Tested pre-commit hook with actual commit - verified it runs all 38 tests
4. Hook successfully prevents commits when tests fail

**Hook Behavior:**
```bash
# On commit attempt:
Running pre-commit hook...
Compiling and running tests...
[runs all 38 tests]
✅ All tests passed! Proceeding with commit.
```

**Note:** Git hooks are local to each repository clone and are not tracked by git. Users cloning this repository will need to set up hooks manually if desired.

## 2025-11-05 15:31:00 AGB - Store Git Hooks in Repository

Store the git hooks on the repository itself, then install them into the .git folder.

### Result

Successfully moved git hooks into the repository for version control and created an installation system:

**Repository Structure:**
```
hooks/
├── README.md        # Documentation for the hooks
├── pre-commit       # Pre-commit hook script
└── pre-push         # Pre-push hook script

install-hooks.sh     # Installation script
```

**Installation Script** (`install-hooks.sh`)
- Automated installation of hooks from `hooks/` directory to `.git/hooks/`
- Validates repository structure before installation
- Sets correct executable permissions automatically
- Provides clear success/error messages
- Lists installed hooks after completion

**Benefits:**
1. **Version Controlled:** Hooks are now tracked in git and shared with all developers
2. **Easy Setup:** New developers can install hooks with a single command: `./install-hooks.sh`
3. **Maintainable:** Updates to hooks are version controlled and can be reviewed in PRs
4. **Documented:** Added `hooks/README.md` with installation and usage instructions

**Installation Process:**
```bash
# After cloning the repository:
./install-hooks.sh

# Output:
Installing git hooks...
  Installing pre-commit...
  Installing pre-push...
✅ Git hooks installed successfully!

Installed hooks:
pre-commit
pre-push
```

**Testing:**
- Verified installation script works correctly
- Confirmed hooks are executable after installation
- Tested pre-commit hook runs all 38 tests before commits
- All functionality working as expected

## 2025-11-05 15:34:58 AGB - GitHub Actions CI/CD Setup

Create a github actions setup to a) compile, b) run tests.

### Result

Successfully implemented GitHub Actions CI/CD workflow for automated testing and compilation:

**Workflow File:** `.github/workflows/ci.yml`

**Triggers:**
- Every push to `master` or `main` branches
- Every pull request targeting `master` or `main` branches

**Workflow Steps:**

1. **Checkout code** - Retrieves latest repository code
2. **Install Rust toolchain** - Sets up stable Rust compiler using `dtolnay/rust-toolchain`
3. **Cache dependencies** - Caches cargo registry, git index, and target directory
   - Significantly speeds up subsequent builds
   - Uses `Cargo.lock` hash as cache key
4. **Install system dependencies** - Installs required GUI libraries
   - libgtk-3-dev, libx11-dev, libxcb (for eframe/egui GUI)
5. **Build project** - Compiles with `cargo build --verbose`
6. **Run tests** - Executes all 38 tests with `cargo test --verbose`
7. **Check formatting** - Validates code style with `cargo fmt --check` (optional, continues on error)
8. **Run clippy** - Performs linting with `cargo clippy` (optional, continues on error)

**Features:**
- **Automatic execution** on every push and PR
- **Fast builds** with intelligent caching
- **Comprehensive testing** runs all 38 tests (21 unit, 11 e2e, 6 integration)
- **Code quality checks** with formatting and clippy
- **Clear feedback** in GitHub Actions UI

**Documentation:**
- Created `.github/README.md` with workflow documentation
- Includes troubleshooting guide and local testing instructions
- Documents how to add CI status badges

**Benefits:**
1. **Automated Quality Assurance:** Every change is automatically tested
2. **Fast Feedback:** Developers know immediately if changes break tests
3. **Protected Branches:** Can require CI to pass before merging PRs
4. **Team Visibility:** All team members can see test results
5. **Prevents Regressions:** Catches issues before they reach main branch

**Verification:**
- Workflow syntax validated
- Local build tested successfully
- All dependencies properly configured
- Ready to run on first push to GitHub

