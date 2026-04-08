//! AEAT text sanitization.
//!
//! The BOE format requires uppercase text in ISO-8859-1, without tildes, but
//! keeping Ñ and Ç which are part of the allowed alphabet.
//!
//! - `sanitize_alpha`      → only letters (A-Z, Ñ, Ç) and spaces
//! - `sanitize_alphanumeric` → letters + digits + spaces
//!
//! Invalid characters are replaced with a space.

/// Map a single source char to its sanitized representation.
/// Returns a `char` (not `Option`) — invalid input becomes `' '`.
fn fold_char(c: char, allow_digit: bool) -> char {
    // Fast path: already ASCII
    if c.is_ascii() {
        let u = c.to_ascii_uppercase();
        if u.is_ascii_uppercase() {
            return u;
        }
        if allow_digit && u.is_ascii_digit() {
            return u;
        }
        if u == ' ' {
            return ' ';
        }
        return ' ';
    }

    // Preserved letters
    match c {
        'Ñ' | 'ñ' => return 'Ñ',
        'Ç' | 'ç' => return 'Ç',
        _ => {}
    }

    // Accent folding (Spanish-relevant cases)
    let folded = match c {
        'Á' | 'À' | 'Ä' | 'Â' | 'Ã' | 'Å' | 'á' | 'à' | 'ä' | 'â' | 'ã' | 'å' => 'A',
        'É' | 'È' | 'Ë' | 'Ê' | 'é' | 'è' | 'ë' | 'ê' => 'E',
        'Í' | 'Ì' | 'Ï' | 'Î' | 'í' | 'ì' | 'ï' | 'î' => 'I',
        'Ó' | 'Ò' | 'Ö' | 'Ô' | 'Õ' | 'ó' | 'ò' | 'ö' | 'ô' | 'õ' => 'O',
        'Ú' | 'Ù' | 'Ü' | 'Û' | 'ú' | 'ù' | 'ü' | 'û' => 'U',
        'Ý' | 'ý' | 'ÿ' => 'Y',
        _ => return ' ',
    };
    let _ = allow_digit;
    folded
}

pub fn sanitize_alpha(s: &str) -> String {
    s.chars().map(|c| fold_char(c, false)).collect()
}

pub fn sanitize_alphanumeric(s: &str) -> String {
    s.chars().map(|c| fold_char(c, true)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uppercases_ascii() {
        assert_eq!(sanitize_alpha("hola mundo"), "HOLA MUNDO");
    }

    #[test]
    fn strips_accents_keeps_enye_and_cedilla() {
        assert_eq!(sanitize_alpha("Señor Peña"), "SEÑOR PEÑA");
        assert_eq!(sanitize_alpha("Barça"), "BARÇA");
        assert_eq!(sanitize_alpha("José Martín"), "JOSE MARTIN");
    }

    #[test]
    fn digits_only_in_alphanumeric() {
        assert_eq!(sanitize_alpha("A1B2"), "A B ");
        assert_eq!(sanitize_alphanumeric("A1B2"), "A1B2");
    }

    #[test]
    fn replaces_invalid_with_space() {
        assert_eq!(sanitize_alpha("hi!"), "HI ");
        assert_eq!(sanitize_alphanumeric("a-b"), "A B");
    }
}
