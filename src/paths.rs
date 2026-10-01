use std::{
    collections::HashSet,
    env,
    error::Error,
    ffi::OsStr,
    fmt, fs,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct ValidatedPaths {
    pub roots: Vec<PathBuf>,
    pub warnings: Vec<PathWarning>,
}

#[derive(Debug)]
pub struct PathWarning {
    pub path: PathBuf,
    pub message: String,
}

impl fmt::Display for PathWarning {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.path.display(), self.message)
    }
}

#[derive(Debug)]
pub enum PathConfigError {
    NoPathsConfigured,
    NoUsableRoots { warnings: Vec<PathWarning> },
}

impl fmt::Display for PathConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoPathsConfigured => {
                formatter.write_str("no documentation paths configured; set MMANPATH or use -M")
            }
            Self::NoUsableRoots { .. } => {
                formatter.write_str("none of the configured documentation paths are usable")
            }
        }
    }
}

impl Error for PathConfigError {}

pub fn validate_roots(paths: &[PathBuf]) -> Result<ValidatedPaths, PathConfigError> {
    if paths.is_empty() {
        return Err(PathConfigError::NoPathsConfigured);
    }

    let mut roots = Vec::new();
    let mut warnings = Vec::new();
    let mut seen = HashSet::new();

    for path in paths {
        let canonical = match fs::canonicalize(path) {
            Ok(canonical) => canonical,
            Err(error) => {
                warnings.push(PathWarning {
                    path: path.clone(),
                    message: error.to_string(),
                });
                continue;
            }
        };

        let metadata = match fs::metadata(&canonical) {
            Ok(metadata) => metadata,
            Err(error) => {
                warnings.push(PathWarning {
                    path: path.clone(),
                    message: error.to_string(),
                });
                continue;
            }
        };

        if !metadata.is_dir() {
            warnings.push(PathWarning {
                path: path.clone(),
                message: String::from("not a directory"),
            });
            continue;
        }

        if let Err(error) = fs::read_dir(&canonical) {
            warnings.push(PathWarning {
                path: path.clone(),
                message: error.to_string(),
            });
            continue;
        }

        if seen.insert(canonical.clone()) {
            roots.push(canonical);
        }
    }

    if roots.is_empty() {
        return Err(PathConfigError::NoUsableRoots { warnings });
    }

    Ok(ValidatedPaths { roots, warnings })
}

pub fn search_paths(
    cli_paths: Option<&OsStr>,
    environment_paths: Option<&OsStr>,
    home: Option<&Path>,
) -> Vec<PathBuf> {
    let Some(value) = cli_paths.or(environment_paths) else {
        return Vec::new();
    };

    let mut paths = Vec::new();
    let mut seen = HashSet::new();

    for path in env::split_paths(value) {
        if path.as_os_str().is_empty() {
            continue;
        }

        let path = expand_tilde(path, home);
        if seen.insert(path.clone()) {
            paths.push(path);
        }
    }

    paths
}

fn expand_tilde(path: PathBuf, home: Option<&Path>) -> PathBuf {
    let Some(home) = home else {
        return path;
    };

    match path.strip_prefix("~") {
        Ok(remainder) => home.join(remainder),
        Err(_) => path,
    }
}

#[cfg(test)]
mod tests {
    use std::{
        env,
        ffi::OsStr,
        fs,
        path::{Path, PathBuf},
    };

    use tempfile::tempdir;

    use super::{PathConfigError, search_paths, validate_roots};

    #[test]
    fn parses_multiple_paths_in_order() {
        let value = env::join_paths(["first", "second"]).expect("test paths should be valid");

        let paths = search_paths(Some(&value), None, None);

        assert_eq!(paths, vec![PathBuf::from("first"), PathBuf::from("second")]);
    }

    #[test]
    fn command_line_paths_replace_environment_paths() {
        let cli = OsStr::new("cli");
        let environment = OsStr::new("environment");

        let paths = search_paths(Some(cli), Some(environment), None);

        assert_eq!(paths, vec![PathBuf::from("cli")]);
    }

    #[test]
    fn ignores_empty_entries() {
        let value = env::join_paths(["first", "", "second"]).expect("test paths should be valid");

        let paths = search_paths(Some(&value), None, None);

        assert_eq!(paths, vec![PathBuf::from("first"), PathBuf::from("second")]);
    }

    #[test]
    fn expands_distinct_tilde_component() {
        let value =
            env::join_paths(["~/manuals", "~other/manuals"]).expect("test paths should be valid");

        let paths = search_paths(Some(&value), None, Some(Path::new("/home/test")));

        assert_eq!(
            paths,
            vec![
                PathBuf::from("/home/test/manuals"),
                PathBuf::from("~other/manuals")
            ]
        );
    }

    #[test]
    fn removes_duplicate_paths_without_reordering() {
        let value =
            env::join_paths(["first", "second", "first"]).expect("test paths should be valid");

        let paths = search_paths(Some(&value), None, None);

        assert_eq!(paths, vec![PathBuf::from("first"), PathBuf::from("second")]);
    }

    #[test]
    fn rejects_missing_configuration() {
        let result = validate_roots(&[]);

        assert!(matches!(result, Err(PathConfigError::NoPathsConfigured)));
    }

    #[test]
    fn accepts_and_canonicalizes_directory() {
        let directory = tempdir().expect("temporary directory shuold be created");

        let validated =
            validate_roots(&[directory.path().to_path_buf()]).expect("directory should be usable");

        assert_eq!(
            validated.roots,
            vec![
                directory
                    .path()
                    .canonicalize()
                    .expect("directory should canonicalize")
            ]
        );

        assert!(validated.warnings.is_empty());
    }

    #[test]
    fn warns_about_invalid_root_and_keeps_valid_root() {
        let directory = tempdir().expect("temporary directory should be created");
        let missing = directory.path().join("missing");

        let validated = validate_roots(&[missing.clone(), directory.path().to_path_buf()])
            .expect("one directory should be usable");

        assert_eq!(validated.roots.len(), 1);
        assert_eq!(validated.warnings.len(), 1);
        assert_eq!(validated.warnings[0].path, missing);
    }

    #[test]
    fn rejects_regular_file_as_root() {
        let directory = tempdir().expect("temporary directory should be created");
        let file = directory.path().join("page.md");

        fs::write(&file, "# Page").expect("test file should be written");

        let result = validate_roots(&[file]);

        assert!(matches!(result, Err(PathConfigError::NoUsableRoots { .. })));
    }

    #[test]
    fn removes_canonical_duplicates() {
        let directory = tempdir().expect("temporary directory should be created");
        let same_directory = directory.path().join(".");

        let validated = validate_roots(&[directory.path().to_path_buf(), same_directory])
            .expect("directory should be usable");

        assert_eq!(validated.roots.len(), 1);
    }
}
