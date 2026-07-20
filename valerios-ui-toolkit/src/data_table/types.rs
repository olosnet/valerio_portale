use leptos::prelude::*;
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq)]
pub enum SortDir {
    None,
    Asc,
    Desc,
}

#[derive(Clone)]
pub struct ColumnDef<T: Clone + 'static> {
    pub title: &'static str,
    pub sortable: bool,
    pub searchable: bool,
    pub cell: Arc<dyn Fn(&T) -> AnyView + Send + Sync>,
    pub sort_key: Option<Arc<dyn Fn(&T) -> String + Send + Sync>>,
    pub search_key: Option<Arc<dyn Fn(&T) -> String + Send + Sync>>,
}

#[derive(Clone)]
pub enum DataTableSource<T: Clone + 'static> {
    Client(Vec<T>),
}

#[derive(Clone, PartialEq)]
pub struct DataTableResponse<T: Clone + PartialEq> {
    pub data: Vec<T>,
    pub total_count: usize,
}

pub struct DataTableSnapshot {
    pub page: usize,
    pub page_size: usize,
    pub sort_field: Option<String>,
    pub sort_dir: SortDir,
    pub search: String,
}
