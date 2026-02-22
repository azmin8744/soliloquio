mod create;
mod delete;
mod read;
mod update;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use models::posts::{self, Column, Model};
use sea_orm::entity::prelude::Uuid;
use sea_orm::sea_query::Expr;
use sea_orm::*;
use serde::{Deserialize, Serialize};

const DEFAULT_PAGE_SIZE: usize = 20;
const MAX_PAGE_SIZE: usize = 100;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PostSortBy {
    CreatedAt,
    UpdatedAt,
    Title,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Serialize, Deserialize)]
struct PostCursor {
    s: String,
    v: String,
    i: Uuid,
}

fn sort_tag(sort_by: &PostSortBy) -> &'static str {
    match sort_by {
        PostSortBy::CreatedAt => "c",
        PostSortBy::UpdatedAt => "u",
        PostSortBy::Title => "t",
    }
}

fn sort_column(sort_by: &PostSortBy) -> Column {
    match sort_by {
        PostSortBy::CreatedAt => Column::CreatedAt,
        PostSortBy::UpdatedAt => Column::UpdatedAt,
        PostSortBy::Title => Column::Title,
    }
}

fn encode_cursor(sort_by: &PostSortBy, post: &Model) -> String {
    let v = match sort_by {
        PostSortBy::CreatedAt => post.created_at.and_utc().to_rfc3339(),
        PostSortBy::UpdatedAt => post.updated_at.and_utc().to_rfc3339(),
        PostSortBy::Title => post.title.clone(),
    };
    let cursor = PostCursor {
        s: sort_tag(sort_by).to_string(),
        v,
        i: post.id,
    };
    let json = serde_json::to_string(&cursor).unwrap();
    URL_SAFE_NO_PAD.encode(json.as_bytes())
}

fn decode_cursor(cursor: &str, expected: &PostSortBy) -> Result<PostCursor, String> {
    let bytes = URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| "Invalid cursor".to_string())?;
    let json = String::from_utf8(bytes).map_err(|_| "Invalid cursor".to_string())?;
    let pc: PostCursor =
        serde_json::from_str(&json).map_err(|_| "Invalid cursor".to_string())?;
    if pc.s != sort_tag(expected) {
        return Err(
            "Cursor sort mismatch: cursor was created with a different sort order".to_string(),
        );
    }
    Ok(pc)
}

fn build_keyset_filter(
    sort_by: &PostSortBy,
    sort_dir: &SortDirection,
    pc: &PostCursor,
) -> Result<Condition, String> {
    let col = sort_column(sort_by);

    let cursor_val: sea_orm::Value = match sort_by {
        PostSortBy::CreatedAt | PostSortBy::UpdatedAt => {
            let dt = chrono::DateTime::parse_from_rfc3339(&pc.v)
                .map_err(|_| "Invalid cursor".to_string())?;
            dt.naive_utc().into()
        }
        PostSortBy::Title => pc.v.clone().into(),
    };

    let (col_cmp, id_cmp): (
        fn(Column, sea_orm::Value) -> sea_orm::sea_query::SimpleExpr,
        fn(Column, Uuid) -> sea_orm::sea_query::SimpleExpr,
    ) = match sort_dir {
        SortDirection::Desc => (
            |c, v| Expr::col((posts::Entity, c)).lt(v),
            |c, id| Expr::col((posts::Entity, c)).lt(id),
        ),
        SortDirection::Asc => (
            |c, v| Expr::col((posts::Entity, c)).gt(v),
            |c, id| Expr::col((posts::Entity, c)).gt(id),
        ),
    };

    Ok(Condition::any()
        .add(col_cmp(col, cursor_val.clone()))
        .add(
            Condition::all()
                .add(Expr::col((posts::Entity, col)).eq(cursor_val))
                .add(id_cmp(Column::Id, pc.i)),
        ))
}

#[derive(Debug)]
pub struct PaginatedPosts {
    pub posts: Vec<Model>,
    pub cursors: Vec<String>,
    pub has_previous_page: bool,
    pub has_next_page: bool,
}

pub struct PostRepository;

#[cfg(test)]
mod slug_tests {
    use super::slugify;

    #[test]
    fn test_slugify_basic() {
        assert_eq!(slugify("Hello World"), "hello-world");
    }

    #[test]
    fn test_slugify_special_chars() {
        assert_eq!(slugify("Rust & Cargo!"), "rust-cargo");
    }

    #[test]
    fn test_slugify_multiple_spaces() {
        assert_eq!(slugify("a  b   c"), "a-b-c");
    }

    #[test]
    fn test_slugify_leading_trailing_special() {
        assert_eq!(slugify("  hello  "), "hello");
    }

    #[test]
    fn test_slugify_empty() {
        assert_eq!(slugify(""), "post");
    }

    #[test]
    fn test_slugify_numbers() {
        assert_eq!(slugify("Post 42"), "post-42");
    }
}

pub(crate) fn slugify(title: &str) -> String {
    let s: String = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    // collapse consecutive hyphens, trim leading/trailing hyphens
    let mut out = String::new();
    let mut prev_hyphen = true; // start true to trim leading
    for c in s.chars() {
        if c == '-' {
            if !prev_hyphen {
                out.push('-');
                prev_hyphen = true;
            }
        } else {
            out.push(c);
            prev_hyphen = false;
        }
    }
    if out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        out.push_str("post");
    }
    out
}
