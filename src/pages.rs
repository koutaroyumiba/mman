use std::{
    collections::BTreeMap,
    error::Error,
    ffi::OsStr,
    fmt, fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopicError {
    Empty,
    Absolute,
    EmptyComponent,
    CurrentDirectory,
    ParentDirectory,
    HiddenComponent,
    ControlCharacter,
}

impl fmt::Display for TopicError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Empty => "topic cannot be empty",
            Self::Absolute => "topic cannot be an absolute path",
            Self::EmptyComponent => "topic cannot contain empty path components",
            Self::CurrentDirectory => "topic cannot contain a current-directory component",
            Self::ParentDirectory => "topic cannot contain a parent-directory component",
            Self::HiddenComponent => "topic cannot contain hidden path components",
            Self::ControlCharacter => "topic cannot contain control characters",
        };

        formatter.write_str(message)
    }
}

impl Error for TopicError {}

pub fn normalize_topic(topic: &str) -> Result<String, TopicError> {
    if topic.is_empty() {
        return Err(TopicError::Empty);
    }

    if topic.starts_with('/') {
        return Err(TopicError::Absolute);
    }

    let topic = topic.strip_suffix(".md").unwrap_or(topic);

    if topic.is_empty() {
        return Err(TopicError::Empty);
    }

    for component in topic.split('/') {
        if component.is_empty() {
            return Err(TopicError::EmptyComponent);
        }

        if component == "." {
            return Err(TopicError::CurrentDirectory);
        }

        if component == ".." {
            return Err(TopicError::ParentDirectory);
        }

        if component.starts_with('.') {
            return Err(TopicError::HiddenComponent);
        }

        if component.chars().any(char::is_control) {
            return Err(TopicError::ControlCharacter);
        }
    }

    Ok(topic.to_owned())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    pub topic: String,
    pub path: PathBuf,
    kind: PageKind,
}

impl Page {
    pub fn read_raw(&self) -> io::Result<Vec<u8>> {
        fs::read(&self.path)
    }
}

// we allow for index pages (ownership/index.md => ownership page)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum PageKind {
    Direct,
    Index,
}

#[derive(Debug)]
pub enum LookupError {
    InvalidTopic(TopicError),
    NotFound(String),
}

impl fmt::Display for LookupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LookupError::InvalidTopic(error) => write!(formatter, "invalid topic: {error}"),
            LookupError::NotFound(topic) => write!(formatter, "no manual page found for {topic}"),
        }
    }
}

impl Error for LookupError {}

/// Searchable collection of all discovered pages
#[derive(Debug, Default)]
pub struct PageIndex {
    pages: BTreeMap<String, Vec<Page>>,
}

impl PageIndex {
    pub fn discover(roots: &[PathBuf]) -> io::Result<Self> {
        let mut pages_by_topic: BTreeMap<String, Vec<Page>> = BTreeMap::new();

        for root in roots {
            for page in discover_pages(root)? {
                pages_by_topic
                    .entry(page.topic.clone())
                    .or_default()
                    .push(page);
            }
        }

        Ok(Self {
            pages: pages_by_topic,
        })
    }

    /// find the default page for an exact topic
    pub fn resolve(&self, topic: &str) -> Option<&Page> {
        self.pages.get(topic)?.first()
    }

    /// Return every discovered copy of one topic
    pub fn resolve_all(&self, topic: &str) -> &[Page] {
        match self.pages.get(topic) {
            Some(page) => page,
            None => &[],
        }
    }

    /// Iterates over unique topics in alphabetical order
    pub fn topics(&self) -> impl Iterator<Item = &str> {
        self.pages.keys().map(|item| item.as_str())
    }

    /// Checks if the index contains any topics
    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }

    /// User-facing operation
    pub fn lookup(&self, topic: &str) -> Result<&Page, LookupError> {
        let normalized_topic = normalize_topic(topic).map_err(LookupError::InvalidTopic)?;

        match self.resolve(normalized_topic.as_str()) {
            Some(page) => Ok(page),
            None => Err(LookupError::NotFound(normalized_topic)),
        }
    }
}

pub fn discover_pages(root: &Path) -> io::Result<Vec<Page>> {
    let mut pages = Vec::new();

    discover_directory(root, root, &mut pages)?;

    pages.sort_by(|left, right| {
        left.topic
            .cmp(&right.topic)
            .then(left.kind.cmp(&right.kind))
            .then(left.path.cmp(&right.path))
    });

    Ok(pages)
}

fn discover_directory(root: &Path, directory: &Path, pages: &mut Vec<Page>) -> io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let name = entry.file_name();

        if name.as_encoded_bytes().starts_with(b".") {
            continue;
        }

        let file_type = entry.file_type()?;
        let path = entry.path();

        if file_type.is_dir() {
            discover_directory(root, &path, pages)?;
            continue;
        }

        let is_page = if file_type.is_file() {
            true
        } else if file_type.is_symlink() {
            entry.metadata()?.is_file()
        } else {
            false
        };

        if !is_page || path.extension() != Some(OsStr::new("md")) {
            continue;
        }

        if let Some((topic, kind)) = topic_from_path(root, &path) {
            pages.push(Page { topic, path, kind });
        }
    }

    Ok(())
}

fn topic_from_path(root: &Path, path: &Path) -> Option<(String, PageKind)> {
    let relative = path.strip_prefix(root).ok()?;

    let (topic_path, kind) = if relative.file_name()? == "index.md" {
        (relative.parent()?.to_path_buf(), PageKind::Index)
    } else {
        (relative.with_extension(""), PageKind::Direct)
    };

    if topic_path.as_os_str().is_empty() {
        return None;
    }

    let components: Option<Vec<&str>> = topic_path
        .components()
        .map(|component| component.as_os_str().to_str())
        .collect();

    Some((components?.join("/"), kind))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{LookupError, PageIndex, TopicError, discover_pages, normalize_topic};

    #[test]
    fn accepts_plain_topic() {
        assert_eq!(normalize_topic("ownership"), Ok(String::from("ownership")));
    }

    #[test]
    fn accepts_nested_topic() {
        assert_eq!(
            normalize_topic("concepts/ownership"),
            Ok(String::from("concepts/ownership"))
        );
    }

    #[test]
    fn removes_markdown_extension() {
        assert_eq!(
            normalize_topic("concepts/ownership.md"),
            Ok(String::from("concepts/ownership"))
        );
    }

    #[test]
    fn preserves_topic_case() {
        assert_eq!(
            normalize_topic("Concepts/Ownership"),
            Ok(String::from("Concepts/Ownership"))
        );
    }

    #[test]
    fn rejects_absolute_topic() {
        assert_eq!(
            normalize_topic("/concepts/ownership"),
            Err(TopicError::Absolute)
        );
    }

    #[test]
    fn rejects_current_directory_component() {
        assert_eq!(
            normalize_topic("./ownership"),
            Err(TopicError::CurrentDirectory)
        );
    }

    #[test]
    fn rejects_parent_directory_component() {
        assert_eq!(
            normalize_topic("concepts/../ownership"),
            Err(TopicError::ParentDirectory)
        );
    }

    #[test]
    fn rejects_hidden_component() {
        assert_eq!(
            normalize_topic("concepts/.private/ownership"),
            Err(TopicError::HiddenComponent)
        );
    }

    #[test]
    fn rejects_repeated_separator() {
        assert_eq!(
            normalize_topic("concepts//ownership"),
            Err(TopicError::EmptyComponent)
        );
    }

    #[test]
    fn rejects_control_characters() {
        assert_eq!(
            normalize_topic("concepts/\u{1b}ownership"),
            Err(TopicError::ControlCharacter)
        );
    }

    #[test]
    fn discovers_root_and_nested_pages() {
        let directory = tempdir().expect("temporary directory should be created");
        let nested = directory.path().join("concepts");

        fs::create_dir(&nested).expect("nested directory should be created");
        fs::write(directory.path().join("git.md"), "# Git").expect("root page should be written");
        fs::write(nested.join("ownership.md"), "# Ownership")
            .expect("nested page should be written");

        let pages = discover_pages(directory.path()).expect("pages should be discovered");

        let topics: Vec<&str> = pages.iter().map(|page| page.topic.as_str()).collect();

        assert_eq!(topics, vec!["concepts/ownership", "git"]);
    }

    #[test]
    fn normalizes_index_page_to_parent_topic() {
        let directory = tempdir().expect("temporary directory should be created");
        let page_directory = directory.path().join("concepts/ownership");

        fs::create_dir_all(&page_directory).expect("page directory should be created");
        fs::write(page_directory.join("index.md"), "# Ownership")
            .expect("index page should be written");

        let pages = discover_pages(directory.path()).expect("pages should be discovered");

        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].topic, "concepts/ownership");
    }

    #[test]
    fn prefers_direct_page_over_index_page() {
        let directory = tempdir().expect("temporary directory should be created");
        let index_directory = directory.path().join("ownership");

        fs::create_dir(&index_directory).expect("index directory should be created");
        fs::write(directory.path().join("ownership.md"), "# Direct")
            .expect("direct page should be written");
        fs::write(index_directory.join("index.md"), "# Index")
            .expect("index page should be written");

        let pages = discover_pages(directory.path()).expect("pages should be discovered");

        assert_eq!(pages.len(), 2);
        assert_eq!(pages[0].path, directory.path().join("ownership.md"));
        assert_eq!(pages[1].path, index_directory.join("index.md"));
    }

    #[test]
    fn ignores_hidden_paths_and_non_markdown_files() {
        let directory = tempdir().expect("temporary directory should be created");
        let hidden_directory = directory.path().join(".private");

        fs::create_dir(&hidden_directory).expect("hidden directory should be created");
        fs::write(directory.path().join(".hidden.md"), "# Hidden")
            .expect("hidden page should be written");
        fs::write(hidden_directory.join("secret.md"), "# Secret")
            .expect("hidden nested page should be written");
        fs::write(directory.path().join("notes.txt"), "Notes")
            .expect("text file should be written");
        fs::write(directory.path().join("visible.md"), "# Visible")
            .expect("visible page should be written");

        let pages = discover_pages(directory.path()).expect("pages should be discovered");

        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].topic, "visible");
    }

    #[test]
    fn ignores_root_index_page() {
        let directory = tempdir().expect("temporary directory should be created");

        fs::write(directory.path().join("index.md"), "# Root")
            .expect("index page should be written");

        let pages = discover_pages(directory.path()).expect("pages should be discovered");

        assert!(pages.is_empty());
    }

    #[test]
    fn page_index_uses_root_order_for_resolution() {
        let first = tempdir().expect("first root should be created");
        let second = tempdir().expect("second root should be created");
        let first_topic_directory = first.path().join("ownership");

        fs::create_dir(&first_topic_directory).expect("topic directory should be created");
        fs::write(first_topic_directory.join("index.md"), "# First")
            .expect("first page should be written");
        fs::write(second.path().join("ownership.md"), "# Second")
            .expect("second page should be written");

        let index = PageIndex::discover(&[first.path().to_path_buf(), second.path().to_path_buf()])
            .expect("page index should be built");

        let resolved = index
            .resolve("ownership")
            .expect("ownership should resolve");

        assert_eq!(resolved.path, first_topic_directory.join("index.md"));
    }

    #[test]
    fn page_index_retains_all_copies_in_precedence_order() {
        let first = tempdir().expect("first root should be created");
        let second = tempdir().expect("second root should be created");

        fs::write(first.path().join("git.md"), "# First").expect("first page should be written");
        fs::write(second.path().join("git.md"), "# Second").expect("second page should be written");

        let index = PageIndex::discover(&[first.path().to_path_buf(), second.path().to_path_buf()])
            .expect("page index should be built");

        let copies = index.resolve_all("git");

        assert_eq!(copies.len(), 2);
        assert_eq!(copies[0].path, first.path().join("git.md"));
        assert_eq!(copies[1].path, second.path().join("git.md"));
    }

    #[test]
    fn page_index_lists_unique_topics_alphabetically() {
        let directory = tempdir().expect("temporary directory should be created");

        fs::write(directory.path().join("vector.md"), "# Vector")
            .expect("vector page should be written");
        fs::write(directory.path().join("algorithm.md"), "# Algorithm")
            .expect("algorithm page should be written");
        fs::write(directory.path().join("git.md"), "# Git").expect("git page should be written");

        let index = PageIndex::discover(&[directory.path().to_path_buf()])
            .expect("page index should be built");
        let topics: Vec<&str> = index.topics().collect();

        assert_eq!(topics, vec!["algorithm", "git", "vector"]);
    }

    #[test]
    fn page_index_resolution_is_case_sensitive() {
        let directory = tempdir().expect("temporary directory should be created");

        fs::write(directory.path().join("Ownership.md"), "# Ownership")
            .expect("page should be written");

        let index = PageIndex::discover(&[directory.path().to_path_buf()])
            .expect("page index should be built");

        assert!(index.resolve("Ownership").is_some());
        assert!(index.resolve("ownership").is_none());
    }

    #[test]
    fn page_index_can_be_empty() {
        let index = PageIndex::discover(&[]).expect("empty page index should be built");

        assert!(index.is_empty());
        assert_eq!(index.topics().count(), 0);
        assert!(index.resolve("missing").is_none());
        assert!(index.resolve_all("missing").is_empty());
    }

    #[test]
    fn lookup_accepts_topic_with_or_without_markdown_extension() {
        let directory = tempdir().expect("temporary directory should be created");
        let page_path = directory.path().join("ownership.md");

        fs::write(&page_path, "# Ownership").expect("page should be written");

        let index = PageIndex::discover(&[directory.path().to_path_buf()])
            .expect("page index should be built");

        assert_eq!(
            index
                .lookup("ownership")
                .expect("topic should resolve")
                .path,
            page_path
        );
        assert_eq!(
            index
                .lookup("ownership.md")
                .expect("topic with extension should resolve")
                .path,
            page_path
        );
    }

    #[test]
    fn lookup_rejects_invalid_topic() {
        let index = PageIndex::discover(&[]).expect("empty page index should be built");

        let result = index.lookup("../ownership");

        assert!(matches!(
            result,
            Err(LookupError::InvalidTopic(TopicError::ParentDirectory))
        ));
    }

    #[test]
    fn lookup_reports_missing_normalized_topic() {
        let index = PageIndex::discover(&[]).expect("empty page index should be built");

        let result = index.lookup("missing.md");

        assert!(matches!(
            result,
            Err(LookupError::NotFound(topic)) if topic == "missing"
        ));
    }

    #[test]
    fn lookup_is_case_sensitive() {
        let directory = tempdir().expect("temporary directory should be created");

        fs::write(directory.path().join("Ownership.md"), "# Ownership")
            .expect("page should be written");

        let index = PageIndex::discover(&[directory.path().to_path_buf()])
            .expect("page index should be built");

        assert!(index.lookup("Ownership").is_ok());
        assert!(matches!(
            index.lookup("ownership"),
            Err(LookupError::NotFound(topic)) if topic == "ownership"
        ));
    }

    #[test]
    fn read_raw_preserves_source_bytes_exactly() {
        let directory = tempdir().expect("temporary directory should be created");
        let page_path = directory.path().join("ownership.md");
        let source = b"# Ownership\r\n\r\nBorrowing notes.\r\n";

        fs::write(&page_path, source).expect("page should be written");

        let index = PageIndex::discover(&[directory.path().to_path_buf()])
            .expect("page index should be built");
        let page = index.lookup("ownership").expect("topic should resolve");

        assert_eq!(page.read_raw().expect("page should be readable"), source);
    }

    #[test]
    fn read_raw_fails_if_page_disappears_after_discovery() {
        let directory = tempdir().expect("temporary directory should be created");
        let page_path = directory.path().join("ownership.md");

        fs::write(&page_path, "# Ownership").expect("page should be written");

        let index = PageIndex::discover(&[directory.path().to_path_buf()])
            .expect("page index should be built");
        let page = index.lookup("ownership").expect("topic should resolve");

        fs::remove_file(&page_path).expect("page should be removed");

        assert!(page.read_raw().is_err());
    }
}
