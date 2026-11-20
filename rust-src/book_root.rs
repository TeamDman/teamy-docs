//! Portable book discovery. Reads are side-effect free; saving is explicit.
use directories_next::ProjectDirs;
use eyre::{Context, ensure};
use facet::Facet;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use teamy_cancellation::CancellationToken;

const SAVED_ROOT_LIMIT: u64 = 65_536;
const SAVED_ROOT_FILE: &str = "book-root.txt";
static SAVE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Result of explicitly saving a canonical book root.
#[derive(Facet, Debug)]
pub struct SavedBookRoot {
    pub book_root: String,
    pub config_path: String,
}

/// Select explicit root, environment, saved preference, or exactly the current directory.
///
/// # Errors
/// A selected invalid root or config is an error; lower-priority roots are not tried.
pub fn resolve_book_root(explicit: Option<&Path>) -> eyre::Result<PathBuf> {
    if let Some(root) = explicit {
        return validate_book_root(root).wrap_err("invalid --book-root");
    }
    if let Some(root) = std::env::var_os("TEAMY_DOCS_BOOK_ROOT") {
        return validate_book_root(Path::new(&root))
            .wrap_err("invalid TEAMY_DOCS_BOOK_ROOT; correct it or use --book-root");
    }
    let config = saved_root_path()?;
    match File::open(&config) {
        Ok(file) => {
            let mut bytes = Vec::new();
            file.take(SAVED_ROOT_LIMIT + 1).read_to_end(&mut bytes)
                .wrap_err_with(|| format!("cannot read saved book root {}", config.display()))?;
            ensure!(bytes.len() as u64 <= SAVED_ROOT_LIMIT, "saved book root exceeds 64 KiB; repair with book-root set --path");
            let text = String::from_utf8(bytes).wrap_err("saved book root is not UTF-8; repair with book-root set --path")?;
            let root = Path::new(text.trim_end_matches(['\r', '\n']));
            ensure!(root.is_absolute(), "saved book root must be an absolute path; repair with book-root set --path");
            validate_book_root(root).wrap_err("saved book root is invalid; repair with book-root set --path or use --book-root")
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            validate_book_root(&std::env::current_dir()?).wrap_err(
                "no default book is configured and the current directory is not a book; run install.ps1, book-root set --path <book>, or use --book-root",
            )
        }
        Err(error) => Err(error).wrap_err_with(|| format!("cannot open saved book root {}; use --book-root to override", config.display())),
    }
}

/// Return the platform config file without creating a directory.
///
/// # Errors
/// Returns invalid config override or unavailable platform-directory errors.
pub fn saved_root_path() -> eyre::Result<PathBuf> {
    let directory = if let Some(directory) = std::env::var_os("TEAMY_DOCS_CONFIG_DIR") {
        let directory = PathBuf::from(directory);
        ensure!(
            directory.is_absolute(),
            "TEAMY_DOCS_CONFIG_DIR must be an absolute, nonempty path"
        );
        directory
    } else {
        ProjectDirs::from("", "TeamDman", "teamy-docs")
            .ok_or_else(|| eyre::eyre!("cannot resolve the user config directory; use TEAMY_DOCS_CONFIG_DIR or --book-root"))?
            .config_dir()
            .to_path_buf()
    };
    Ok(directory.join(SAVED_ROOT_FILE))
}

/// Save an existing, canonical book root, replacing the previous preference.
///
/// # Errors
/// Returns invalid book, cancellation, UTF-8 path, or file-write errors.
pub fn save_book_root(
    root: &Path,
    cancellation: &CancellationToken,
) -> eyre::Result<SavedBookRoot> {
    cancellation.bail_if_cancelled()?;
    let root = validate_book_root(root)?;
    let root_text = root
        .to_str()
        .ok_or_else(|| eyre::eyre!("book root must be a UTF-8 path to save it"))?;
    ensure!(
        !root_text.contains(['\r', '\n']),
        "book root cannot contain line breaks"
    );
    ensure!(
        (root_text.len() as u64) < SAVED_ROOT_LIMIT,
        "book root exceeds 64 KiB"
    );
    let config = saved_root_path()?;
    let directory = config
        .parent()
        .ok_or_else(|| eyre::eyre!("config path has no parent"))?;
    cancellation.bail_if_cancelled()?;
    fs::create_dir_all(directory).wrap_err("cannot create the user config directory")?;
    let sequence = SAVE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let temporary = directory.join(format!(".book-root-{}-{sequence}.tmp", std::process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let result = (|| -> eyre::Result<()> {
        file.write_all(root_text.as_bytes())?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        drop(file);
        cancellation.bail_if_cancelled()?;
        fs::rename(&temporary, &config).wrap_err("cannot replace the saved book root")?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result?;
    if std::env::var_os("TEAMY_DOCS_BOOK_ROOT").is_some() {
        tracing::warn!("TEAMY_DOCS_BOOK_ROOT overrides the saved book root");
    }
    Ok(SavedBookRoot {
        book_root: root_text.to_owned(),
        config_path: config.to_string_lossy().into_owned(),
    })
}

fn validate_book_root(root: &Path) -> eyre::Result<PathBuf> {
    ensure!(!root.as_os_str().is_empty(), "book root cannot be empty");
    let root = root.canonicalize().wrap_err("cannot resolve book root")?;
    ensure!(root.is_dir(), "book root is not a directory");
    let config = root
        .join("book.toml")
        .canonicalize()
        .wrap_err("book root must contain book.toml")?;
    ensure!(
        config.starts_with(&root) && config.is_file(),
        "book.toml must be a file contained in the book root"
    );
    Ok(root)
}
