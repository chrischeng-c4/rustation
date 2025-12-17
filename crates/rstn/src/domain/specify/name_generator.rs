//! Feature name generation
//!
//! Converts user descriptions to kebab-case names suitable for
//! branch names and directory names.

/// Stop words to filter out from descriptions
const STOP_WORDS: &[&str] = &[
    "i", "a", "an", "the", "to", "for", "of", "in", "on", "at", "by", "with", "from", "is", "are",
    "was", "were", "be", "been", "being", "have", "has", "had", "do", "does", "did", "will",
    "would", "should", "could", "can", "may", "might", "must", "shall", "this", "that", "these",
    "those", "my", "your", "our", "their", "want", "need", "add", "get", "set",
];

/// Generate a kebab-case feature name from a description
///
/// # Algorithm
///
/// 1. Convert to lowercase
/// 2. Split on non-alphanumeric characters
/// 3. Filter out stop words and short words (<3 chars)
/// 4. Take first 3-4 meaningful words
/// 5. Join with hyphens
/// 6. Truncate to max length at word boundary
///
/// # Arguments
///
/// * `description` - The user's feature description
/// * `max_length` - Maximum length for the generated name (default: 50)
///
/// # Returns
///
/// A kebab-case string suitable for branch/directory names
pub fn generate_feature_name(description: &str, max_length: usize) -> String {
    let words: Vec<String> = description
        .to_lowercase()
        // Split on any non-alphanumeric character
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        // Filter out stop words
        .filter(|word| !STOP_WORDS.contains(word))
        // Filter out short words (less than 3 chars)
        .filter(|word| word.len() >= 3)
        .map(String::from)
        .collect();

    // Take first 3-4 meaningful words
    let max_words = if words.len() == 4 { 4 } else { 3 };
    let selected: Vec<&str> = words.iter().take(max_words).map(|s| s.as_str()).collect();

    let mut result = selected.join("-");

    // Truncate at word boundary if too long
    if result.len() > max_length {
        result = truncate_at_word_boundary(&result, max_length);
    }

    // Clean up: remove leading/trailing hyphens, collapse multiple hyphens
    result = result
        .trim_matches('-')
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    result
}

/// Extract a human-readable title from the description
///
/// Takes the first sentence or first 100 characters, whichever is shorter.
/// Capitalizes appropriately for display.
///
/// # Arguments
///
/// * `description` - The user's feature description
///
/// # Returns
///
/// A title-cased string suitable for display
pub fn extract_title(description: &str) -> String {
    // Take first sentence (up to period, exclamation, or question mark)
    let first_sentence = description
        .split(['.', '!', '?'])
        .next()
        .unwrap_or(description)
        .trim();

    // Limit to 100 characters
    let truncated = if first_sentence.len() > 100 {
        let boundary = find_word_boundary(first_sentence, 100);
        &first_sentence[..boundary]
    } else {
        first_sentence
    };

    // Title case: capitalize first letter of each word
    truncated
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Truncate a string at a word boundary
fn truncate_at_word_boundary(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }

    // Find the last hyphen before the limit
    let truncated = &s[..max_len];
    if let Some(last_hyphen) = truncated.rfind('-') {
        s[..last_hyphen].to_string()
    } else {
        truncated.to_string()
    }
}

/// Find a word boundary near the given position
fn find_word_boundary(s: &str, near: usize) -> usize {
    if near >= s.len() {
        return s.len();
    }

    // Look backwards for a space
    let search_area = &s[..near];
    if let Some(pos) = search_area.rfind(' ') {
        pos
    } else {
        near
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_description() {
        let result = generate_feature_name("Add user authentication", 50);
        assert_eq!(result, "user-authentication");
    }

    #[test]
    fn test_with_stop_words() {
        let result = generate_feature_name("I want to implement OAuth2 login", 50);
        assert_eq!(result, "implement-oauth2-login");
    }

    #[test]
    fn test_with_special_characters() {
        let result = generate_feature_name("Add user/admin authentication!", 50);
        assert_eq!(result, "user-admin-authentication");
    }

    #[test]
    fn test_long_description_truncation() {
        let result = generate_feature_name(
            "Implement comprehensive user authentication system with OAuth2 support",
            30,
        );
        assert!(result.len() <= 30);
        assert!(!result.ends_with('-'));
    }

    #[test]
    fn test_extract_title_simple() {
        let result = extract_title("Add user authentication with OAuth2");
        assert_eq!(result, "Add User Authentication With OAuth2");
    }

    #[test]
    fn test_extract_title_with_sentence() {
        let result = extract_title("Add user auth. This is the second sentence.");
        assert_eq!(result, "Add User Auth");
    }

    #[test]
    fn test_numbers_preserved() {
        let result = generate_feature_name("Implement OAuth2 with PKCE", 50);
        assert!(result.contains("oauth2"));
        assert!(result.contains("pkce"));
    }

    #[test]
    fn test_empty_after_filtering() {
        // All stop words should result in empty or minimal output
        let result = generate_feature_name("I want to add the", 50);
        assert!(result.is_empty() || result.len() < 5);
    }
}
