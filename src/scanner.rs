//! Line scanner for keyword matches.
//!
//! Pure Rust — no nvim-oxi dependency.  Scans a line of text and returns
//! every comment-keyword match with its byte offset.

use crate::keywords::Keyword;

/// A single keyword match within a line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    /// The matched keyword.
    pub keyword: Keyword,
    /// Byte offset of the keyword within the line.
    pub byte_offset: usize,
    /// Byte length of the keyword tag (e.g. `4` for `TODO`, `5` for `FIXME`).
    pub byte_len: usize,
}

/// Scan a single line for keyword matches.
///
/// Keywords are recognised only after a comment leader character sequence
/// (`///`, `//!`, `//`, `/*`, `--`, `#`, `*`, `%`, `;`) or at the very
/// start of a line (after whitespace).  The `"` leader is only recognised
/// at the start of a line (Vim convention).  This avoids false positives
/// inside string literals and identifiers.
#[must_use]
pub fn scan_line(line: &str) -> Vec<Match> {
    let mut results = Vec::new();

    // Find positions where a keyword search is warranted — right after a
    // comment leader or at the very start of the line.
    for start in keyword_candidate_positions(line) {
        let rest = &line[start..];
        if let Some(kw) = Keyword::match_at(rest) {
            results.push(Match {
                keyword: kw,
                byte_offset: start,
                byte_len: kw.name().len(),
            });
        }
    }

    results
}

/// Yield byte offsets within `line` where a keyword could plausibly begin.
///
/// We look for positions immediately after a comment leader token, skipping
/// any interstitial whitespace.  We also consider the start of the line
/// (after leading whitespace) if it begins with a comment leader.
fn keyword_candidate_positions(line: &str) -> Vec<usize> {
    let mut positions = Vec::new();
    let trimmed = line.trim_start();
    let leading_ws = line.len() - trimmed.len();

    // Check if the start of the line (after whitespace) is a keyword directly.
    if !trimmed.is_empty() && Keyword::match_at(trimmed).is_some() {
        positions.push(leading_ws);
    }

    // Comment leaders, ordered longest-first so `///` and `//!` match before
    // `//`, and `/*` before `*`.
    //
    // `"` is only valid as a Vim comment leader at the start of a line, so we
    // handle it separately below instead of putting it in the general list.
    let leaders: &[&str] = &["///", "//!", "//", "/*", "--", "#", "*", "%", ";"];

    let len = line.len();
    let mut i = 0;

    while i < len {
        let mut matched_leader = false;
        for leader in leaders {
            let llen = leader.len();
            if i + llen <= len && &line[i..i + llen] == *leader {
                // Skip past leader and any trailing `/` for doc-comment
                // variants like `////`.
                let mut after_leader = i + llen;
                // For `//`-family leaders, consume any additional slashes.
                if leader.starts_with("//") {
                    while after_leader < len
                        && line.as_bytes()[after_leader] == b'/'
                    {
                        after_leader += 1;
                    }
                }
                // Skip whitespace after leader.
                let pos = skip_whitespace(line, after_leader);
                if pos < len && !positions.contains(&pos) {
                    positions.push(pos);
                }
                i = after_leader;
                matched_leader = true;
                break;
            }
        }
        if !matched_leader {
            i += 1;
        }
    }

    // Handle `"` as start-of-line comment leader (Vim convention).
    if trimmed.starts_with('"') {
        let after_quote = leading_ws + 1;
        let pos = skip_whitespace(line, after_quote);
        if pos < len && !positions.contains(&pos) {
            positions.push(pos);
        }
    }

    positions
}

/// Skip ASCII whitespace starting at `from`, returning the first non-space
/// byte offset (or `line.len()` if the rest is whitespace).
fn skip_whitespace(line: &str, from: usize) -> usize {
    let bytes = line.as_bytes();
    let mut i = from;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    i
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- helpers ------------------------------------------------------------

    fn first_match(line: &str) -> Option<Match> {
        scan_line(line).into_iter().next()
    }

    fn assert_keyword(line: &str, expected: Keyword) {
        let m = first_match(line);
        assert!(
            m.is_some(),
            "expected keyword {expected} in line: {line:?}"
        );
        assert_eq!(m.unwrap().keyword, expected);
    }

    fn assert_no_match(line: &str) {
        let matches = scan_line(line);
        assert!(
            matches.is_empty(),
            "expected no match in line: {line:?}, got: {matches:?}"
        );
    }

    // -- basic detection ----------------------------------------------------

    #[test]
    fn detects_todo_with_double_slash() {
        assert_keyword("// TODO: fix this", Keyword::Todo);
    }

    #[test]
    fn detects_fixme_with_hash() {
        assert_keyword("# FIXME: broken", Keyword::Fixme);
    }

    #[test]
    fn detects_hack_with_block_comment() {
        assert_keyword("/* HACK: workaround */", Keyword::Hack);
    }

    #[test]
    fn detects_warn_with_double_dash() {
        assert_keyword("-- WARN: deprecated", Keyword::Warn);
    }

    #[test]
    fn detects_note_with_star() {
        assert_keyword(" * NOTE: important", Keyword::Note);
    }

    #[test]
    fn detects_perf_with_percent() {
        assert_keyword("% PERF: slow query", Keyword::Perf);
    }

    #[test]
    fn detects_test_with_semicolon() {
        assert_keyword("; TEST: edge case", Keyword::Test);
    }

    // -- case insensitivity -------------------------------------------------

    #[test]
    fn case_insensitive_detection() {
        assert_keyword("// todo: lower", Keyword::Todo);
        assert_keyword("// Todo: mixed", Keyword::Todo);
        assert_keyword("# fixme: lower", Keyword::Fixme);
    }

    // -- optional colon -----------------------------------------------------

    #[test]
    fn keyword_without_colon() {
        assert_keyword("// TODO fix this", Keyword::Todo);
    }

    #[test]
    fn keyword_with_colon_no_space() {
        assert_keyword("// TODO:fix this", Keyword::Todo);
    }

    // -- parenthesised attribution ------------------------------------------

    #[test]
    fn keyword_with_paren_attribution() {
        assert_keyword("// TODO(user): clean up", Keyword::Todo);
    }

    // -- indented comment ---------------------------------------------------

    #[test]
    fn indented_comment() {
        assert_keyword("    // TODO: indented", Keyword::Todo);
    }

    // -- at start of line (bare keyword) ------------------------------------

    #[test]
    fn bare_keyword_at_start() {
        assert_keyword("TODO: standalone", Keyword::Todo);
    }

    #[test]
    fn bare_keyword_with_leading_whitespace() {
        assert_keyword("   FIXME: indented bare", Keyword::Fixme);
    }

    // -- no false positives ------------------------------------------------

    #[test]
    fn no_match_in_identifier() {
        // "TODOISH" is not a keyword — alpha after the tag.
        assert_no_match("let todoish = 5;");
    }

    #[test]
    fn no_match_in_string_literal() {
        // No comment leader, and the start-of-line trim yields `let`.
        assert_no_match("let s = \"TODO: not a comment\";");
    }

    #[test]
    fn no_match_plain_code() {
        assert_no_match("fn main() {}");
    }

    #[test]
    fn no_match_empty_line() {
        assert_no_match("");
    }

    #[test]
    fn no_match_whitespace_only() {
        assert_no_match("   ");
    }

    // -- byte offsets -------------------------------------------------------

    #[test]
    fn byte_offset_is_correct() {
        let m = first_match("// TODO: fix").unwrap();
        assert_eq!(m.byte_offset, 3); // "// " → offset 3
        assert_eq!(m.byte_len, 4); // "TODO"
    }

    #[test]
    fn byte_offset_with_extra_spaces() {
        let m = first_match("//   FIXME: blah").unwrap();
        assert_eq!(m.byte_offset, 5); // "//   " → offset 5
        assert_eq!(m.byte_len, 5); // "FIXME"
    }

    // -- multiple keywords in one line --------------------------------------

    #[test]
    fn multiple_keywords_in_line() {
        let matches = scan_line("// TODO: fix this HACK: workaround");
        // At minimum we should find TODO (the first one after the comment leader).
        assert!(matches.iter().any(|m| m.keyword == Keyword::Todo));
    }

    // -- vim double-quote comment leader ------------------------------------

    #[test]
    fn vim_double_quote_comment() {
        assert_keyword("\" TODO: vimscript comment", Keyword::Todo);
    }

    // -- Rust doc comment ---------------------------------------------------

    #[test]
    fn rust_doc_comment() {
        assert_keyword("/// TODO: document this", Keyword::Todo);
    }

    // -- no keyword after leader but has one later --------------------------

    #[test]
    fn keyword_deep_in_comment() {
        // "// some text TODO:" — the TODO is after the comment leader's text.
        // We should still pick it up since the start-of-content search finds
        // "some" first, but the `//` leader search finds "some" too.
        // Actually: `keyword_candidate_positions` only finds the position right
        // after the leader.  So "// some text TODO:" would not match via the
        // leader path.  That is intentional — we only highlight keywords at the
        // beginning of comment content, matching todo-comments.nvim behaviour.
        let matches = scan_line("// some text TODO: late");
        // This might or might not match depending on design.  In our current
        // implementation, the candidate position after "//" is "some", which
        // does not match.  So we expect no match here.
        assert!(matches.is_empty());
    }
}
