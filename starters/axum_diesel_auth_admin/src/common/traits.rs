use convert_case::{Case, Casing};
pub use macros::Displayable;

pub trait Displayable {
    type SqlType;
    type BoxedQuery<'a>;

    fn all() -> Self::BoxedQuery<'static>;

    fn paginated<'a>(
        offset: i64,
        page_size: i64,
        search_term: Option<String>,
    ) -> Self::BoxedQuery<'a>;

    fn table_headers() -> Vec<&'static str>;
    fn table_titles() -> Vec<String> {
        let headers = Self::table_headers();
        headers.iter().map(|h| h.to_case(Case::Title)).collect()
    }
    fn to_table_row(&self) -> Vec<String>;
}
