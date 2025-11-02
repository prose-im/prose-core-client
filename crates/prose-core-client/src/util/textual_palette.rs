use unicode_normalization::UnicodeNormalization;

/// Color palette for textual avatars
const TEXTUAL_PALETTE_COLORS: &[&str] = &[
    "df74c9", "05cd8f", "52a6db", "ee733d", "f48686", "6b6f8c", "e13030", "8e30de", "b258ec",
    "f15e5e", "3159ea", "7ab0ff", "78c670", "18aeec", "8125d4", "c32ea3", "415dae", "d79b25",
    "ce811a", "2ba032",
];

/// Normalizes textual initials by converting to uppercase and removing accents/diacritics
/// This function performs Unicode NFD normalization and filters out combining diacritical marks.
pub fn normalize_textual_initials(initials: impl AsRef<str>) -> String {
    initials
        .as_ref()
        .to_uppercase()
        .nfd()
        .filter(|&c| {
            // Filter out combining diacritical marks
            !('\u{0300}'..='\u{036F}').contains(&c)
        })
        .collect()
}

/// Generates a color from the textual palette based on a string value
pub fn generate_textual_palette(value: &str) -> String {
    // Compute value fingerprint
    let value_fingerprint: u32 = value.chars().map(|c| c as u32).sum();

    // Acquire color based on value fingerprint
    let color_index = (value_fingerprint as usize) % TEXTUAL_PALETTE_COLORS.len();
    let color = TEXTUAL_PALETTE_COLORS[color_index];

    format!("#{}", color)
}

/// Generates textual initials from a JID-like string and/or name
pub fn generate_textual_initials(name: &str) -> Option<String> {
    let name_chunks: Vec<&str> = name
        .split(' ')
        .map(|chunk| chunk.trim())
        .filter(|chunk| !chunk.is_empty())
        .collect();

    // Extract first name and family name initials?
    if name_chunks.len() >= 2 {
        let first_initial = name_chunks[0].chars().next()?;
        let second_initial = name_chunks[1].chars().next()?;
        return Some(format!("{}{}", first_initial, second_initial));
    }

    // Extract first two characters of first name?
    if !name_chunks.is_empty() && name_chunks[0].len() >= 1 {
        let first_chunk = name_chunks[0];
        let initials: String = first_chunk.chars().take(2).collect();
        return Some(initials);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_textual_initials() {
        assert_eq!(&normalize_textual_initials("ab"), "AB");
        assert_eq!(&normalize_textual_initials("àé"), "AE");
        assert_eq!(&normalize_textual_initials("Ñoël"), "NOEL");
        assert_eq!(&normalize_textual_initials(""), "");
    }

    #[test]
    fn test_generate_textual_palette() {
        let color = generate_textual_palette("foo");
        assert!(color.starts_with('#'));
        assert_eq!(color.len(), 7);

        assert_eq!(
            generate_textual_palette("foo"),
            generate_textual_palette("foo")
        );
        assert_ne!(
            generate_textual_palette("foo"),
            generate_textual_palette("bar")
        );
    }

    #[test]
    fn test_generate_textual_initials() {
        assert_eq!(generate_textual_initials("John Doe").as_deref(), Some("JD"));
        assert_eq!(generate_textual_initials("John").as_deref(), Some("Jo"));
        assert_eq!(generate_textual_initials(""), None);
        assert_eq!(
            generate_textual_initials("  John   Doe  ").as_deref(),
            Some("JD")
        );
        assert_eq!(
            generate_textual_initials("John Michael Doe").as_deref(),
            Some("JM")
        );
    }
}
