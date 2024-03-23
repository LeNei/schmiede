use serde::Deserialize;

fn default_page_size() -> Option<u32> {
    Some(10)
}

fn default_page() -> Option<u32> {
    Some(1)
}

#[derive(Deserialize)]
pub struct TablePaginationParams {
    #[serde(default = "default_page")]
    pub page: Option<u32>, // The current page number (for pagination)
    #[serde(default = "default_page_size")]
    pub page_size: Option<u32>, // Number of items to display per page
    pub search_term: Option<String>, // Optional string for full-text search
}

impl TablePaginationParams {
    pub fn to_query_params(&self) -> (i64, i64, Option<String>) {
        let page = self.page.unwrap_or(1) as i64 - 1; // Subtract 1 to convert to 0-indexed page
        let page_size = self.page_size.unwrap_or(10) as i64;
        let offset = page * page_size;
        (offset, page_size, self.search_term.clone())
    }
}
