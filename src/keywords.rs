//! Keyword definitions with colors and sign text.
//!
//! Pure Rust — no nvim-oxi dependency. Defines the vocabulary of comment
//! keywords that shirube recognises and their visual properties.

/// A keyword that shirube highlights in comments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Keyword {
    Todo,
    Fixme,
    Hack,
    Warn,
    Note,
    Perf,
    Test,
}

impl Keyword {
    /// All supported keywords.
    pub const ALL: &[Self] = &[
        Self::Todo,
        Self::Fixme,
        Self::Hack,
        Self::Warn,
        Self::Note,
        Self::Perf,
        Self::Test,
    ];

    /// The canonical display name (uppercase).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Todo => "TODO",
            Self::Fixme => "FIXME",
            Self::Hack => "HACK",
            Self::Warn => "WARN",
            Self::Note => "NOTE",
            Self::Perf => "PERF",
            Self::Test => "TEST",
        }
    }

    /// Foreground color (hex) for the highlight group.
    #[must_use]
    pub const fn fg_color(self) -> &'static str {
        match self {
            Self::Todo => "#2563eb",  // blue
            Self::Fixme => "#dc2626", // red
            Self::Hack => "#ea580c",  // orange
            Self::Warn => "#d97706",  // amber
            Self::Note => "#16a34a",  // green
            Self::Perf => "#9333ea",  // purple
            Self::Test => "#0891b2",  // cyan
        }
    }

    /// Background color (hex) — a faint tint behind the keyword text.
    #[must_use]
    pub const fn bg_color(self) -> &'static str {
        match self {
            Self::Todo => "#1e3a5f",
            Self::Fixme => "#3f1219",
            Self::Hack => "#431407",
            Self::Warn => "#451a03",
            Self::Note => "#052e16",
            Self::Perf => "#2e1065",
            Self::Test => "#083344",
        }
    }

    /// The highlight group name used in Neovim (e.g. `ShirubeTodo`).
    #[must_use]
    pub const fn hl_group(self) -> &'static str {
        match self {
            Self::Todo => "ShirubeTodo",
            Self::Fixme => "ShirubeFixme",
            Self::Hack => "ShirubeHack",
            Self::Warn => "ShirubeWarn",
            Self::Note => "ShirubeNote",
            Self::Perf => "ShirubePerf",
            Self::Test => "ShirubeTest",
        }
    }

    /// Sign-column text (two characters max).
    #[must_use]
    pub const fn sign_text(self) -> &'static str {
        match self {
            Self::Todo => "TD",
            Self::Fixme => "FX",
            Self::Hack => "HK",
            Self::Warn => "WN",
            Self::Note => "NT",
            Self::Perf => "PF",
            Self::Test => "TS",
        }
    }

    /// Try to match a keyword tag at the beginning of `s` (case-insensitive).
    ///
    /// Returns `Some(keyword)` if `s` starts with a keyword tag optionally
    /// followed by `:` or whitespace.
    #[must_use]
    pub fn match_at(s: &str) -> Option<Self> {
        for kw in Self::ALL {
            let name = kw.name();
            let len = name.len();
            if s.len() >= len && s[..len].eq_ignore_ascii_case(name) {
                // Accept bare keyword, keyword followed by `:`, `(`, or
                // whitespace.  Reject `TODOISH` (keyword followed by more
                // alpha chars).
                let rest = &s[len..];
                if rest.is_empty()
                    || rest.starts_with(':')
                    || rest.starts_with('(')
                    || rest
                        .chars()
                        .next()
                        .is_some_and(|c| c.is_ascii_whitespace())
                {
                    return Some(*kw);
                }
            }
        }
        None
    }
}

impl std::fmt::Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_keywords_have_unique_hl_groups() {
        let mut groups: Vec<&str> = Keyword::ALL.iter().map(|k| k.hl_group()).collect();
        groups.sort_unstable();
        groups.dedup();
        assert_eq!(groups.len(), Keyword::ALL.len());
    }

    #[test]
    fn all_keywords_have_unique_sign_text() {
        let mut signs: Vec<&str> = Keyword::ALL.iter().map(|k| k.sign_text()).collect();
        signs.sort_unstable();
        signs.dedup();
        assert_eq!(signs.len(), Keyword::ALL.len());
    }

    #[test]
    fn sign_text_max_two_chars() {
        for kw in Keyword::ALL {
            assert!(
                kw.sign_text().len() <= 2,
                "{} sign_text too long",
                kw.name()
            );
        }
    }

    #[test]
    fn match_at_exact() {
        assert_eq!(Keyword::match_at("TODO"), Some(Keyword::Todo));
        assert_eq!(Keyword::match_at("FIXME"), Some(Keyword::Fixme));
        assert_eq!(Keyword::match_at("HACK"), Some(Keyword::Hack));
        assert_eq!(Keyword::match_at("WARN"), Some(Keyword::Warn));
        assert_eq!(Keyword::match_at("NOTE"), Some(Keyword::Note));
        assert_eq!(Keyword::match_at("PERF"), Some(Keyword::Perf));
        assert_eq!(Keyword::match_at("TEST"), Some(Keyword::Test));
    }

    #[test]
    fn match_at_case_insensitive() {
        assert_eq!(Keyword::match_at("todo"), Some(Keyword::Todo));
        assert_eq!(Keyword::match_at("Todo"), Some(Keyword::Todo));
        assert_eq!(Keyword::match_at("fixme"), Some(Keyword::Fixme));
        assert_eq!(Keyword::match_at("Fixme"), Some(Keyword::Fixme));
    }

    #[test]
    fn match_at_with_colon() {
        assert_eq!(Keyword::match_at("TODO:"), Some(Keyword::Todo));
        assert_eq!(Keyword::match_at("TODO: fix this"), Some(Keyword::Todo));
        assert_eq!(Keyword::match_at("fixme: urgent"), Some(Keyword::Fixme));
    }

    #[test]
    fn match_at_with_paren() {
        assert_eq!(
            Keyword::match_at("TODO(drzzln): check"),
            Some(Keyword::Todo)
        );
    }

    #[test]
    fn match_at_with_whitespace() {
        assert_eq!(
            Keyword::match_at("TODO fix this later"),
            Some(Keyword::Todo)
        );
    }

    #[test]
    fn match_at_rejects_continuation() {
        // "TODOISH" should not match — the keyword is followed by more alpha.
        assert_eq!(Keyword::match_at("TODOISH"), None);
        assert_eq!(Keyword::match_at("FIXMEUP"), None);
        assert_eq!(Keyword::match_at("NOTING"), None);
    }

    #[test]
    fn match_at_empty() {
        assert_eq!(Keyword::match_at(""), None);
    }

    #[test]
    fn match_at_no_keyword() {
        assert_eq!(Keyword::match_at("hello world"), None);
        assert_eq!(Keyword::match_at("let x = 5;"), None);
    }

    #[test]
    fn display_impl() {
        assert_eq!(format!("{}", Keyword::Todo), "TODO");
        assert_eq!(format!("{}", Keyword::Fixme), "FIXME");
    }

    #[test]
    fn fg_colors_are_valid_hex() {
        for kw in Keyword::ALL {
            let c = kw.fg_color();
            assert!(c.starts_with('#'), "{} fg_color missing #", kw.name());
            assert_eq!(c.len(), 7, "{} fg_color wrong length", kw.name());
        }
    }

    #[test]
    fn bg_colors_are_valid_hex() {
        for kw in Keyword::ALL {
            let c = kw.bg_color();
            assert!(c.starts_with('#'), "{} bg_color missing #", kw.name());
            assert_eq!(c.len(), 7, "{} bg_color wrong length", kw.name());
        }
    }
}
