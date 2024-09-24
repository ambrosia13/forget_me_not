use std::path::Path;

use regex::Regex;

pub fn resolve_includes(source: &mut str, parent_dir: &Path) -> Result<(), std::io::Error> {
    let regex = Regex::new(r"#include (?<path>\w+)!").unwrap();

    for captures in regex.captures_iter(source) {
        let include_path = &captures["path"];
        let include_path = parent_dir.join(include_path);

        let mut include_source = std::fs::read_to_string(include_path)?;
        resolve_includes(&mut include_source, parent_dir)?;
    }

    Ok(())
}
