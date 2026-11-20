use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use teamy_docs::sections::SectionsReport;

static SEQUENCE: AtomicU64 = AtomicU64::new(0);
#[derive(Debug)]
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "teamy-docs-discovery-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
    fn config(&self) -> PathBuf {
        self.0.join("config")
    }
    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_teamy-docs"));
        command
            .current_dir(&self.0)
            .env_remove("TEAMY_DOCS_BOOK_ROOT")
            .env("TEAMY_DOCS_CONFIG_DIR", self.config());
        command
    }
    fn saved(&self, bytes: &[u8]) {
        fs::create_dir_all(self.config()).unwrap();
        fs::write(self.config().join("book-root.txt"), bytes).unwrap();
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/book")
}
fn decode(output: &Output) -> SectionsReport {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    facet_json::from_slice(&output.stdout).unwrap()
}
fn failed(output: &Output, fragment: &str) {
    assert!(!output.status.success(), "unexpected success");
    assert!(
        output.stdout.is_empty(),
        "failed discovery emitted a report"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(fragment),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
#[test]
fn saved_default_works_from_neutral_directory_for_both_aliases() {
    let scratch = Scratch::new();
    let set = scratch
        .command()
        .args(["book-root", "set", "--path"])
        .arg(fixture())
        .output()
        .unwrap();
    assert!(
        set.status.success(),
        "{}",
        String::from_utf8_lossy(&set.stderr)
    );
    assert!(scratch.config().join("book-root.txt").is_file());
    let explicit = scratch
        .command()
        .args(["section", "list", "--book-root"])
        .arg(fixture())
        .output()
        .unwrap();
    let expected = decode(&explicit);
    for name in ["section", "sections"] {
        let output = scratch.command().args([name, "list"]).output().unwrap();
        assert_eq!(decode(&output), expected);
        assert_eq!(output.stdout, explicit.stdout);
    }
    // Replacement must work for an existing preference, including on Windows.
    let set_again = scratch
        .command()
        .args(["book-root", "set", "--path"])
        .arg(fixture())
        .output()
        .unwrap();
    assert!(
        set_again.status.success(),
        "{}",
        String::from_utf8_lossy(&set_again.stderr)
    );
}
#[test]
fn explicit_and_environment_roots_take_precedence_without_reading_bad_config() {
    let scratch = Scratch::new();
    scratch.saved(b"invalid saved root");
    let explicit = scratch
        .command()
        .env("TEAMY_DOCS_BOOK_ROOT", "missing")
        .env("TEAMY_DOCS_CONFIG_DIR", "relative-invalid")
        .args(["section", "list", "--book-root"])
        .arg(fixture())
        .output()
        .unwrap();
    let expected = decode(&explicit);
    let environment = scratch
        .command()
        .env("TEAMY_DOCS_BOOK_ROOT", fixture())
        .env("TEAMY_DOCS_CONFIG_DIR", "relative-invalid")
        .args(["section", "list"])
        .output()
        .unwrap();
    assert_eq!(decode(&environment), expected);
    let invalid_explicit = scratch
        .command()
        .env("TEAMY_DOCS_BOOK_ROOT", fixture())
        .args(["section", "list", "--book-root", "missing"])
        .output()
        .unwrap();
    failed(&invalid_explicit, "invalid --book-root");
}
#[test]
fn missing_default_is_actionable_and_read_only() {
    let scratch = Scratch::new();
    let output = scratch
        .command()
        .args(["section", "list"])
        .output()
        .unwrap();
    failed(&output, "no default book is configured");
    assert!(!scratch.config().exists());
    let exact_cwd = scratch
        .command()
        .current_dir(fixture())
        .args(["section", "list"])
        .output()
        .unwrap();
    decode(&exact_cwd);
    assert!(!scratch.config().exists());
}
#[test]
fn selected_invalid_preferences_do_not_fall_back_to_a_valid_current_book() {
    let scratch = Scratch::new();
    let env_invalid = scratch
        .command()
        .current_dir(fixture())
        .env("TEAMY_DOCS_BOOK_ROOT", "missing")
        .args(["sections", "list"])
        .output()
        .unwrap();
    failed(&env_invalid, "invalid TEAMY_DOCS_BOOK_ROOT");
    let env_empty = scratch
        .command()
        .current_dir(fixture())
        .env("TEAMY_DOCS_BOOK_ROOT", "")
        .args(["section", "list"])
        .output()
        .unwrap();
    failed(&env_empty, "invalid TEAMY_DOCS_BOOK_ROOT");
    let missing = scratch.0.join("stale-root");
    scratch.saved(missing.to_str().unwrap().as_bytes());
    let stale = scratch
        .command()
        .current_dir(fixture())
        .args(["section", "list"])
        .output()
        .unwrap();
    failed(&stale, "saved book root is invalid");
    for (bytes, fragment) in [
        (Vec::new(), "saved book root must be an absolute path"),
        (
            b"relative-root".to_vec(),
            "saved book root must be an absolute path",
        ),
        (vec![0xff], "saved book root is not UTF-8"),
        (vec![b'x'; 65_537], "saved book root exceeds 64 KiB"),
    ] {
        scratch.saved(&bytes);
        let output = scratch
            .command()
            .current_dir(fixture())
            .args(["section", "list"])
            .output()
            .unwrap();
        failed(&output, fragment);
    }
    let bad_config = scratch
        .command()
        .current_dir(fixture())
        .env("TEAMY_DOCS_CONFIG_DIR", "")
        .args(["section", "list"])
        .output()
        .unwrap();
    failed(&bad_config, "TEAMY_DOCS_CONFIG_DIR must be an absolute");
}
#[test]
fn setter_rejects_missing_books_without_creating_preferences() {
    let scratch = Scratch::new();
    let output = scratch
        .command()
        .args(["book-root", "set", "--path", "."])
        .output()
        .unwrap();
    failed(&output, "book root must contain book.toml");
    assert!(!scratch.config().exists());
}
