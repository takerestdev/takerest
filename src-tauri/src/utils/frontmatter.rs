use crate::error::AppError;

/// Parse a markdown file with YAML frontmatter into (yaml_str, body_str).
/// Frontmatter is delimited by `---` on its own line at the start of the file.
///
/// Example input:
/// ```text
/// ---
/// method: GET
/// url: /api/users
/// ---
/// # My Request
/// Some notes here.
/// ```
///
/// Returns ("method: GET\nurl: /api/users\n", "# My Request\nSome notes here.\n")
pub fn parse(content: &str) -> Result<(String, String), AppError> {
    let trimmed = content.trim_start();

    if !trimmed.starts_with("---") {
        return Err(AppError::Other(
            "File does not start with YAML frontmatter delimiter '---'".to_string(),
        ));
    }

    // Skip the opening "---" line
    let after_opening = &trimmed[3..];
    let after_opening = after_opening.strip_prefix('\n').unwrap_or(
        after_opening.strip_prefix("\r\n").unwrap_or(after_opening),
    );

    // Find the closing "---"
    if let Some(end_pos) = find_closing_delimiter(after_opening) {
        let yaml_str = after_opening[..end_pos].to_string();
        let rest = &after_opening[end_pos + 3..]; // skip the "---"
        // Strip the newline after the closing delimiter
        let body = rest
            .strip_prefix('\n')
            .unwrap_or(rest.strip_prefix("\r\n").unwrap_or(rest));
        Ok((yaml_str, body.to_string()))
    } else {
        Err(AppError::Other(
            "Could not find closing frontmatter delimiter '---'".to_string(),
        ))
    }
}

/// Find the position of the closing `---` delimiter.
/// It must be at the start of a line.
fn find_closing_delimiter(content: &str) -> Option<usize> {
    // Check if content starts with ---
    if content.starts_with("---") {
        return Some(0);
    }

    // Search for \n--- or \r\n---
    for (i, _) in content.match_indices('\n') {
        let rest = &content[i + 1..];
        if rest.starts_with("---") {
            return Some(i + 1);
        }
    }

    None
}

/// Serialize YAML frontmatter string and a markdown body into a complete file.
pub fn serialize(yaml_str: &str, body: &str) -> String {
    let mut result = String::new();
    result.push_str("---\n");
    result.push_str(yaml_str);
    if !yaml_str.ends_with('\n') {
        result.push('\n');
    }
    result.push_str("---\n");
    if !body.is_empty() {
        result.push_str(body);
        if !body.ends_with('\n') {
            result.push('\n');
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let input = "---\nmethod: GET\nurl: /api/users\n---\n# My Request\n";
        let (yaml, body) = parse(input).unwrap();
        assert_eq!(yaml, "method: GET\nurl: /api/users\n");
        assert_eq!(body, "# My Request\n");
    }

    #[test]
    fn test_parse_empty_body() {
        let input = "---\nmethod: POST\n---\n";
        let (yaml, body) = parse(input).unwrap();
        assert_eq!(yaml, "method: POST\n");
        assert_eq!(body, "");
    }

    #[test]
    fn test_serialize_roundtrip() {
        let yaml = "method: GET\nurl: /api/users\n";
        let body = "# My Request\nSome notes.\n";
        let output = serialize(yaml, body);
        let (parsed_yaml, parsed_body) = parse(&output).unwrap();
        assert_eq!(parsed_yaml, yaml);
        assert_eq!(parsed_body, body);
    }

    #[test]
    fn test_no_frontmatter() {
        let input = "# Just markdown\nNo frontmatter here.";
        assert!(parse(input).is_err());
    }
}
