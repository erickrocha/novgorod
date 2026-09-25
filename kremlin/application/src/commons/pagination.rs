use utoipa::{IntoParams, ToSchema};

pub const DEFAULT_PAGE: u64 = 1;
pub const DEFAULT_PAGE_SIZE: u64 = 25;
pub const MIN_PAGE_SIZE: u64 = 1;
pub const MAX_PAGE_SIZE: u64 = 100;

#[derive(Debug, Clone, serde::Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct PageQuery {
    pub page: Option<u64>,
    #[serde(alias = "page_size")]
    pub page_size: Option<u64>,
    pub q: Option<String>,
    #[serde(alias = "sort_by")]
    pub sort_by: Option<String>,
    #[serde(alias = "sort_dir")]
    pub sort_dir: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl SortDirection {
    pub fn from_optional_str(s: Option<&str>) -> Self {
        match s.map(|v| v.trim().to_ascii_lowercase()) {
            Some(ref v) if v == "desc" => SortDirection::Desc,
            _ => SortDirection::Asc,
        }
    }

    pub fn is_descending(&self) -> bool {
        matches!(self, SortDirection::Desc)
    }
}

#[derive(Debug, Clone)]
pub struct NormalizedPagination {
    pub page: u64,
    pub page_size: u64,
    pub offset: u64,
    pub q: Option<String>,
    pub sort_by: String,
    pub sort_dir: SortDirection,
}

impl NormalizedPagination {
    pub fn new(query: &PageQuery, allowed_sort_fields: &[&str], default_sort: &str) -> Self {
        let page = query.page.unwrap_or(DEFAULT_PAGE).max(1);
        let page_size = query
            .page_size
            .unwrap_or(DEFAULT_PAGE_SIZE)
            .clamp(MIN_PAGE_SIZE, MAX_PAGE_SIZE);
        let offset = (page - 1) * page_size;

        let q = query
            .q
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToString::to_string);

        let sort_dir = SortDirection::from_optional_str(query.sort_dir.as_deref());

        let sort_by = query
            .sort_by
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .and_then(|candidate| {
                allowed_sort_fields
                    .iter()
                    .find(|&&allowed| allowed.eq_ignore_ascii_case(candidate))
                    .copied()
            })
            .unwrap_or(default_sort)
            .to_string();

        Self {
            page,
            page_size,
            offset,
            q,
            sort_by,
            sort_dir,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PagedResponse<T> {
    pub items: Vec<T>,
    pub total: Option<u64>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub next_cursor: Option<i64>,
}

impl<T> PagedResponse<T> {
    pub fn new(items: Vec<T>, total: u64, page: u64, page_size: u64) -> Self {
        Self {
            items,
            total: Some(total),
            page: Some(page),
            page_size: Some(page_size),
            next_cursor: None,
        }
    }

    pub fn empty(page: u64, page_size: u64) -> Self {
        Self {
            items: Vec::new(),
            total: Some(0),
            page: Some(page),
            page_size: Some(page_size),
            next_cursor: None,
        }
    }

    pub fn page_by_cursor(items: Vec<T>, next_cursor: Option<i64>) -> Self {
        Self {
            items,
            total: None,
            page: None,
            page_size: None,
            next_cursor,
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/commons/pagination.rs"]
mod tests;
