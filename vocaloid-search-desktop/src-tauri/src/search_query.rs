//! Search text syntax shared with the scraper's snapshot query.
//!
//! Mirrors the Niconico snapshot `q` grammar so the three list views and the
//! sync box accept the same input:
//!
//! | input      | meaning                                                    |
//! |------------|------------------------------------------------------------|
//! | `a b`      | `a` AND `b`                                                |
//! | `a OR b`   | `a` OR `b` (OR groups bind before AND: `a OR b c` is `(a OR b) AND c`) |
//! | `"a b"`    | literal `a b` (quotes keep spaces and operators)           |
//! | `-a`       | exclude `a` (no space between `-` and the term)            |
//! | `- a`      | literal `-` and `a`                                        |
//! | `*`        | match everything (local-only); `"*"` and `a*` are literal  |
//!
//! A query made only of excluded terms matches nothing, which is what the
//! snapshot API returns for that form, so it is rendered as a predicate that
//! matches no rows.
//!
//! The `*` wildcard is a local extension: the snapshot API has no wildcard
//! (`q=*` does not mean "everything"), so it must never be sent to that API.
//! `snapshot_query_issue` reports the queries the API cannot express, letting
//! the sync box refuse them instead of silently syncing a wrong subset.

#[derive(Debug, Clone, PartialEq)]
struct Leaf {
    text: String,
    negated: bool,
    /// A bare, unquoted `*`; matches every row locally.
    wildcard: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum Parsed {
    /// No usable term at all.
    Empty,
    /// Only excluded terms; matches nothing (the snapshot API returns no hits
    /// for this form).
    OnlyExcluded,
    /// OR groups that are ANDed together.
    Terms(Vec<Vec<Leaf>>),
}

/// SQL predicate plus the parameters it expects, in order.
#[derive(Debug, Clone, PartialEq)]
pub struct TextMatch {
    pub sql: String,
    pub params: Vec<String>,
}

impl TextMatch {
    pub fn is_empty(&self) -> bool {
        self.sql.is_empty()
    }
}

/// Why a query cannot be sent to the scraper's snapshot `q` API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotQueryIssue {
    /// The query uses the local-only `*` wildcard, which the API cannot express.
    Wildcard,
    /// The query only excludes terms; the API returns no hits for it.
    OnlyExcluded,
}

impl SnapshotQueryIssue {
    /// Stable machine code for the frontend: `"wildcard"` | `"only_excluded"`.
    pub fn code(self) -> &'static str {
        match self {
            SnapshotQueryIssue::Wildcard => "wildcard",
            SnapshotQueryIssue::OnlyExcluded => "only_excluded",
        }
    }

    /// English command-error message (repo rule: Rust error messages are English).
    pub fn message(self) -> &'static str {
        match self {
            SnapshotQueryIssue::Wildcard => {
                "the snapshot API does not support the '*' wildcard; remove it or use a keyword"
            }
            SnapshotQueryIssue::OnlyExcluded => {
                "the snapshot API returns no results for a query that only excludes terms; add at least one keyword"
            }
        }
    }
}

/// `Some(issue)` when `query` must not be sent to the snapshot `q` API.
pub fn snapshot_query_issue(query: &str) -> Option<SnapshotQueryIssue> {
    let has_wildcard =
        tokenize(query)
            .iter()
            .any(|token| matches!(token, Token::Leaf(leaf) if leaf.wildcard));

    if has_wildcard {
        return Some(SnapshotQueryIssue::Wildcard);
    }

    match parse(query) {
        Parsed::OnlyExcluded => Some(SnapshotQueryIssue::OnlyExcluded),
        Parsed::Empty | Parsed::Terms(_) => None,
    }
}

/// Escape a user term so it can be embedded in a `LIKE ... ESCAPE '\'` pattern.
pub fn like_pattern(text: &str) -> String {
    let escaped = text
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("%{}%", escaped)
}

/// Render `query` into a `LIKE ... ESCAPE '\'` predicate over `columns`.
pub fn render_like(query: &str, columns: &[&str]) -> TextMatch {
    render(query, |term| {
        let pattern = like_pattern(term);
        let sql = columns
            .iter()
            .map(|column| format!("{} LIKE ? ESCAPE '\\'", column))
            .collect::<Vec<_>>()
            .join(" OR ");
        TextMatch {
            sql: format!("({})", sql),
            params: vec![pattern; columns.len()],
        }
    })
}

/// Render `query` into a SQL predicate.
///
/// `leaf` renders the positive form of a single term; negation is applied here
/// so every backend shares the same `NOT` behaviour. A `*` wildcard leaf is
/// rendered as `1` without calling `leaf`, since it matches every row locally.
pub fn render<F>(query: &str, mut leaf: F) -> TextMatch
where
    F: FnMut(&str) -> TextMatch,
{
    match parse(query) {
        Parsed::Empty => TextMatch {
            sql: String::new(),
            params: Vec::new(),
        },
        Parsed::OnlyExcluded => TextMatch {
            sql: "0".to_string(),
            params: Vec::new(),
        },
        Parsed::Terms(groups) => {
            let mut params = Vec::new();
            let mut group_sql = Vec::new();

            for group in groups {
                let mut or_sql = Vec::new();
                for leaf_term in group {
                    let rendered = if leaf_term.wildcard {
                        TextMatch {
                            sql: "1".to_string(),
                            params: Vec::new(),
                        }
                    } else {
                        leaf(&leaf_term.text)
                    };
                    params.extend(rendered.params);
                    or_sql.push(if leaf_term.negated {
                        format!("NOT ({})", rendered.sql)
                    } else {
                        rendered.sql
                    });
                }
                if or_sql.len() > 1 {
                    group_sql.push(format!("({})", or_sql.join(" OR ")));
                } else if let Some(single) = or_sql.into_iter().next() {
                    group_sql.push(single);
                }
            }

            TextMatch {
                sql: group_sql.join(" AND "),
                params,
            }
        }
    }
}

enum Token {
    Or,
    Leaf(Leaf),
}

fn parse(query: &str) -> Parsed {
    let tokens = tokenize(query);

    let mut groups: Vec<Vec<Leaf>> = Vec::new();
    let mut pending_or = false;

    for token in tokens {
        match token {
            Token::Or => pending_or = true,
            Token::Leaf(leaf) => {
                if pending_or && !groups.is_empty() {
                    if let Some(last) = groups.last_mut() {
                        last.push(leaf);
                    }
                } else {
                    groups.push(vec![leaf]);
                }
                pending_or = false;
            }
        }
    }

    if groups.is_empty() {
        return Parsed::Empty;
    }

    let has_positive = groups
        .iter()
        .flatten()
        .any(|leaf| !leaf.negated);

    if has_positive {
        Parsed::Terms(groups)
    } else {
        Parsed::OnlyExcluded
    }
}

fn tokenize(query: &str) -> Vec<Token> {
    let chars: Vec<char> = query.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        if chars[i].is_whitespace() {
            i += 1;
            continue;
        }

        if is_or_operator(&chars, i) {
            tokens.push(Token::Or);
            i += 2;
            continue;
        }

        let mut negated = false;
        if chars[i] == '-' && i + 1 < chars.len() && !chars[i + 1].is_whitespace() {
            negated = true;
            i += 1;
        }

        let mut text = String::new();
        let mut wildcard = false;
        if chars[i] == '"' {
            i += 1;
            while i < chars.len() && chars[i] != '"' {
                text.push(chars[i]);
                i += 1;
            }
            if i < chars.len() {
                i += 1;
            }
        } else {
            while i < chars.len() && !chars[i].is_whitespace() {
                text.push(chars[i]);
                i += 1;
            }
            wildcard = text == "*";
        }

        if !text.is_empty() {
            tokens.push(Token::Leaf(Leaf {
                text,
                negated,
                wildcard,
            }));
        }
    }

    tokens
}

/// `OR` is an operator only when it stands alone between spaces.
fn is_or_operator(chars: &[char], i: usize) -> bool {
    if chars[i] != 'O' || chars.get(i + 1) != Some(&'R') {
        return false;
    }
    let before_ok = i == 0 || chars[i - 1].is_whitespace();
    let after_ok = i + 2 >= chars.len() || chars[i + 2].is_whitespace();
    before_ok && after_ok
}

#[cfg(test)]
mod tests {
    use super::{
        like_pattern, render, render_like, snapshot_query_issue, SnapshotQueryIssue, TextMatch,
    };

    fn leaf(text: &str) -> TextMatch {
        TextMatch {
            sql: format!("L({})", text),
            params: vec![text.to_string()],
        }
    }

    fn sql(query: &str) -> String {
        render(query, leaf).sql
    }

    #[test]
    fn treats_empty_query_as_no_filter() {
        let rendered = render("   ", leaf);

        assert!(rendered.is_empty());
        assert!(rendered.params.is_empty());
    }

    #[test]
    fn space_separated_terms_are_anded() {
        assert_eq!(sql("VOCALOID ミク"), "L(VOCALOID) AND L(ミク)");
    }

    #[test]
    fn or_groups_bind_before_and() {
        assert_eq!(sql("ボーカル OR 歌ってみた ミク"), "(L(ボーカル) OR L(歌ってみた)) AND L(ミク)");
    }

    #[test]
    fn quotes_keep_spaces_and_operators() {
        assert_eq!(sql("\"初音 ミク\""), "L(初音 ミク)");
        assert_eq!(sql("\"OR\""), "L(OR)");
    }

    #[test]
    fn leading_dash_excludes_the_term() {
        assert_eq!(sql("VOCALOID -初音ミク"), "L(VOCALOID) AND NOT (L(初音ミク))");
    }

    #[test]
    fn dash_followed_by_space_is_a_literal() {
        assert_eq!(sql("- ボーカル"), "L(-) AND L(ボーカル)");
    }

    #[test]
    fn only_excluded_terms_match_nothing() {
        let rendered = render("-初音ミク", leaf);

        assert_eq!(rendered.sql, "0");
        assert!(rendered.params.is_empty());
    }

    #[test]
    fn params_follow_sql_order() {
        let rendered = render("a OR b -c", leaf);

        assert_eq!(rendered.sql, "(L(a) OR L(b)) AND NOT (L(c))");
        assert_eq!(rendered.params, vec!["a", "b", "c"]);
    }

    #[test]
    fn bare_star_matches_everything() {
        let rendered = render("*", leaf);

        assert_eq!(rendered.sql, "1");
        assert!(rendered.params.is_empty());
    }

    #[test]
    fn star_combines_with_exclusion() {
        assert_eq!(sql("* -初音ミク"), "1 AND NOT (L(初音ミク))");
    }

    #[test]
    fn quoted_or_embedded_star_stays_literal() {
        assert_eq!(sql("\"*\""), "L(*)");
        assert_eq!(sql("VOCALOID*"), "L(VOCALOID*)");
    }

    #[test]
    fn negated_star_matches_nothing() {
        let rendered = render("-*", leaf);

        assert_eq!(rendered.sql, "0");
        assert!(rendered.params.is_empty());
    }

    #[test]
    fn snapshot_query_issue_flags_wildcard_and_only_excluded() {
        assert_eq!(
            snapshot_query_issue("*"),
            Some(SnapshotQueryIssue::Wildcard)
        );
        assert_eq!(
            snapshot_query_issue("* -foo"),
            Some(SnapshotQueryIssue::Wildcard)
        );
        assert_eq!(
            snapshot_query_issue("-foo"),
            Some(SnapshotQueryIssue::OnlyExcluded)
        );
        assert_eq!(snapshot_query_issue(""), None);
        assert_eq!(snapshot_query_issue("foo"), None);
        assert_eq!(snapshot_query_issue("\"*\""), None);
    }

    #[test]
    fn snapshot_query_issue_codes_and_messages_are_stable() {
        assert_eq!(SnapshotQueryIssue::Wildcard.code(), "wildcard");
        assert_eq!(SnapshotQueryIssue::OnlyExcluded.code(), "only_excluded");
        assert!(SnapshotQueryIssue::Wildcard.message().contains("wildcard"));
        assert!(SnapshotQueryIssue::OnlyExcluded.message().contains("excludes"));
    }

    #[test]
    fn like_pattern_escapes_wildcards() {
        assert_eq!(like_pattern("50%_a\\b"), "%50\\%\\_a\\\\b%");
    }

    #[test]
    fn renders_single_column_like_predicate() {
        let rendered = render_like("ミク", &["title"]);

        assert_eq!(rendered.sql, "(title LIKE ? ESCAPE '\\')");
        assert_eq!(rendered.params, vec!["%ミク%"]);
    }

    #[test]
    fn renders_like_predicate_across_columns_with_exclusion() {
        let rendered = render_like("VOCALOID -初音ミク", &["v.title", "v.tags"]);

        assert!(rendered
            .sql
            .contains("(v.title LIKE ? ESCAPE '\\' OR v.tags LIKE ? ESCAPE '\\')"));
        assert!(rendered.sql.contains("NOT ("));
        assert_eq!(
            rendered.params,
            vec!["%VOCALOID%", "%VOCALOID%", "%初音ミク%", "%初音ミク%"]
        );
    }

    #[test]
    fn renders_no_predicate_for_blank_query() {
        assert!(render_like("  ", &["title"]).is_empty());
    }
}
