// Módulo de Cabeçalho de Coluna (ColumnHeader) — Padrão Excel
// Converte índices numéricos para identificadores alfabéticos (A, B, ..., Z, AA, AB, ...) e gerencia metadados do cabeçalho.

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnHeader {
    pub logical_index: usize,
    pub letter: String,
    pub key: String,
    pub title: String,
    pub width: f32,
    pub visible: bool,
    pub is_sorted: bool,
    pub sort_ascending: bool,
    pub has_filter: bool,
}

impl ColumnHeader {
    pub fn new(logical_index: usize, key: &str, title: &str, width: f32) -> Self {
        Self {
            logical_index,
            letter: col_index_to_letter(logical_index),
            key: key.to_string(),
            title: title.to_string(),
            width: width.max(40.0),
            visible: true,
            is_sorted: false,
            sort_ascending: true,
            has_filter: false,
        }
    }
}

/// Converte índice de coluna (0-based) para notação alfabética Excel (A, B, ..., Z, AA, AB, ..., AZ, BA, ...)
pub fn col_index_to_letter(mut n: usize) -> String {
    let mut result = Vec::new();
    loop {
        let rem = n % 26;
        result.push((b'A' + rem as u8) as char);
        if n < 26 {
            break;
        }
        n = (n / 26) - 1;
    }
    result.reverse();
    result.into_iter().collect()
}

/// Converte identificador alfabético Excel para índice numérico (0-based)
pub fn letter_to_col_index(s: &str) -> Option<usize> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let mut result: usize = 0;
    for c in s.chars() {
        if !c.is_ascii_alphabetic() {
            return None;
        }
        let val = (c.to_ascii_uppercase() as u8 - b'A' + 1) as usize;
        result = result.checked_mul(26)?.checked_add(val)?;
    }
    result.checked_sub(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_col_index_to_letter_basic() {
        assert_eq!(col_index_to_letter(0), "A");
        assert_eq!(col_index_to_letter(1), "B");
        assert_eq!(col_index_to_letter(25), "Z");
        assert_eq!(col_index_to_letter(26), "AA");
        assert_eq!(col_index_to_letter(27), "AB");
        assert_eq!(col_index_to_letter(51), "AZ");
        assert_eq!(col_index_to_letter(52), "BA");
        assert_eq!(col_index_to_letter(53), "BB");
        assert_eq!(col_index_to_letter(701), "ZZ");
        assert_eq!(col_index_to_letter(702), "AAA");
    }

    #[test]
    fn test_letter_to_col_index_roundtrip() {
        for idx in [0, 1, 25, 26, 27, 51, 52, 701, 702, 1000] {
            let letter = col_index_to_letter(idx);
            let parsed = letter_to_col_index(&letter).expect("Falha ao parsear coluna");
            assert_eq!(parsed, idx, "Inconsistência para índice {}", idx);
        }
    }

    #[test]
    fn test_letter_to_col_index_case_insensitive() {
        assert_eq!(letter_to_col_index("a"), Some(0));
        assert_eq!(letter_to_col_index("z"), Some(25));
        assert_eq!(letter_to_col_index("aa"), Some(26));
        assert_eq!(letter_to_col_index("Az"), Some(51));
        assert_eq!(letter_to_col_index("123"), None);
        assert_eq!(letter_to_col_index(""), None);
    }
}
