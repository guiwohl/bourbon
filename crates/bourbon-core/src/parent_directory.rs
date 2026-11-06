use std::path::Path;

pub(crate) fn parent_directory_of(path: &Path) -> &Path {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_nested_path_has_its_parent() {
        assert_eq!(parent_directory_of(Path::new("a/b/c.db")), Path::new("a/b"));
    }

    #[test]
    fn a_bare_file_name_lives_in_the_current_directory() {
        assert_eq!(parent_directory_of(Path::new("c.db")), Path::new("."));
    }

    #[test]
    fn the_root_has_no_parent_so_it_is_the_current_directory() {
        assert_eq!(parent_directory_of(Path::new("/")), Path::new("."));
    }
}
