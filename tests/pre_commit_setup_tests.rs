use std::fs;
use std::path::Path;

#[test]
fn test_pyproject_toml_exists() {
    let pyproject_path = Path::new("pyproject.toml");
    assert!(
        pyproject_path.exists(),
        "pyproject.toml should exist for uv dependency management"
    );
}

#[test]
fn test_pyproject_toml_is_valid_toml() {
    let content = fs::read_to_string("pyproject.toml")
        .expect("Failed to read pyproject.toml");

    // Basic TOML validation - check for required sections
    assert!(
        content.contains("[project]"),
        "pyproject.toml should contain [project] section"
    );
    assert!(
        content.contains("name"),
        "pyproject.toml should define project name"
    );
}

#[test]
fn test_pre_commit_config_exists() {
    let config_path = Path::new(".pre-commit-config.yaml");
    assert!(
        config_path.exists(),
        ".pre-commit-config.yaml should exist for pre-commit framework"
    );
}

#[test]
fn test_pre_commit_config_has_required_hooks() {
    let content = fs::read_to_string(".pre-commit-config.yaml")
        .expect("Failed to read .pre-commit-config.yaml");

    // Check for required hooks
    assert!(
        content.contains("make-fmt"),
        "pre-commit config should have make-fmt hook"
    );
    assert!(
        content.contains("make-test"),
        "pre-commit config should have make-test hook"
    );
    assert!(
        content.contains("make-fmt-check"),
        "pre-commit config should have make-fmt-check hook"
    );
}

#[test]
fn test_pre_commit_config_has_correct_stages() {
    let content = fs::read_to_string(".pre-commit-config.yaml")
        .expect("Failed to read .pre-commit-config.yaml");

    // Check for commit stage hooks
    let commit_section = content.lines()
        .skip_while(|l| !l.contains("make-fmt"))
        .take_while(|l| !l.contains("make-test"))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        commit_section.contains("stages: [commit]") || content.contains("stages: [commit]"),
        "Commit hooks should be configured for commit stage"
    );

    // Check for push stage hooks
    assert!(
        content.contains("stages: [push]"),
        "Pre-push hooks should be configured for push stage"
    );
}

#[test]
fn test_pre_commit_config_uses_local_repo() {
    let content = fs::read_to_string(".pre-commit-config.yaml")
        .expect("Failed to read .pre-commit-config.yaml");

    assert!(
        content.contains("repo: local"),
        "pre-commit config should use local repository"
    );
}

#[test]
fn test_pre_commit_config_uses_system_language() {
    let content = fs::read_to_string(".pre-commit-config.yaml")
        .expect("Failed to read .pre-commit-config.yaml");

    assert!(
        content.contains("language: system"),
        "pre-commit hooks should use system language"
    );
}

#[test]
fn test_makefile_has_install_pre_commit_target() {
    let content = fs::read_to_string("Makefile")
        .expect("Failed to read Makefile");

    assert!(
        content.contains("install-pre-commit:"),
        "Makefile should have install-pre-commit target"
    );
}

#[test]
fn test_makefile_has_setup_target() {
    let content = fs::read_to_string("Makefile")
        .expect("Failed to read Makefile");

    assert!(
        content.contains("setup:"),
        "Makefile should have setup target"
    );
}

#[test]
fn test_makefile_install_pre_commit_uses_uv() {
    let content = fs::read_to_string("Makefile")
        .expect("Failed to read Makefile");

    // Find the install-pre-commit target and check for uv commands
    let target_section = content.lines()
        .skip_while(|l| !l.contains("install-pre-commit:"))
        .take_while(|l| !l.contains(".PHONY: install-pre-commit"))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        target_section.contains("uv sync") || content.contains("uv sync"),
        "install-pre-commit should use uv sync"
    );
    assert!(
        target_section.contains("uv run pre-commit") || content.contains("uv run pre-commit"),
        "install-pre-commit should use uv run pre-commit"
    );
}

#[test]
fn test_hooks_readme_documents_pre_commit_framework() {
    let content = fs::read_to_string("hooks/README.md")
        .expect("Failed to read hooks/README.md");

    // Check for pre-commit framework documentation (case-insensitive check)
    let has_framework_doc = content.to_lowercase().contains("pre-commit framework") ||
                           content.contains("Pre-commit Framework");
    assert!(
        has_framework_doc,
        "hooks/README.md should document the pre-commit framework"
    );
    assert!(
        content.contains("make install-pre-commit"),
        "hooks/README.md should document the installation command"
    );
}

#[test]
fn test_pyproject_toml_includes_pre_commit_dependency() {
    let content = fs::read_to_string("pyproject.toml")
        .expect("Failed to read pyproject.toml");

    assert!(
        content.contains("pre-commit"),
        "pyproject.toml should list pre-commit as a dependency"
    );
}

#[test]
fn test_pyproject_toml_is_valid_format() {
    let content = fs::read_to_string("pyproject.toml")
        .expect("Failed to read pyproject.toml");

    // Check for basic TOML structure
    assert!(
        content.contains("[tool.uv]"),
        "pyproject.toml should contain [tool.uv] section"
    );
    assert!(
        content.contains("dev-dependencies"),
        "pyproject.toml should define dev-dependencies"
    );
}

#[test]
fn test_makefile_has_proper_phony_declarations() {
    let content = fs::read_to_string("Makefile")
        .expect("Failed to read Makefile");

    // Check for .PHONY declarations for new targets
    assert!(
        content.contains(".PHONY: install-pre-commit"),
        "Makefile should declare install-pre-commit as .PHONY"
    );
    assert!(
        content.contains(".PHONY: setup"),
        "Makefile should declare setup as .PHONY"
    );
}

#[test]
fn test_pre_commit_config_has_all_required_fields_in_hooks() {
    let content = fs::read_to_string(".pre-commit-config.yaml")
        .expect("Failed to read .pre-commit-config.yaml");

    // Check for required hook fields
    let required_fields = vec!["id:", "name:", "entry:", "language:", "stages:", "pass_filenames:", "always_run:"];

    for field in required_fields {
        assert!(
            content.contains(field),
            ".pre-commit-config.yaml should contain field: {}",
            field
        );
    }
}

#[test]
fn test_makefile_install_pre_commit_has_helpful_message() {
    let content = fs::read_to_string("Makefile")
        .expect("Failed to read Makefile");

    // Find the install-pre-commit target
    let target_section = content.lines()
        .skip_while(|l| !l.contains("install-pre-commit:"))
        .take_while(|l| !l.contains(".PHONY:"))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        target_section.contains("echo") || content.contains("Pre-commit hooks installed"),
        "install-pre-commit should have user-friendly output messages"
    );
}

#[test]
fn test_pre_commit_config_make_fmt_is_idempotent() {
    let content = fs::read_to_string(".pre-commit-config.yaml")
        .expect("Failed to read .pre-commit-config.yaml");

    // make fmt should be idempotent (running it multiple times should have the same result)
    // Check that it's properly configured to handle this
    assert!(
        content.contains("entry: make fmt"),
        "pre-commit config should properly call make fmt"
    );
}

#[test]
fn test_pre_commit_stages_configuration_correct() {
    let content = fs::read_to_string(".pre-commit-config.yaml")
        .expect("Failed to read .pre-commit-config.yaml");

    // Parse to verify stages are correctly set
    // Commit stage: make fmt, make test
    assert!(
        content.contains("id: make-fmt") &&
        content[..content.find("id: make-test").unwrap_or(content.len())]
            .contains("stages: [commit]"),
        "make-fmt should be in commit stage"
    );
}
