#![allow(dead_code)]
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

#[derive(Clone)]
pub struct DataTableResponse<T: Clone> {
    pub data: Vec<T>,
    pub total_count: usize,
}

impl<T: Clone> PartialEq for DataTableResponse<T> {
    fn eq(&self, other: &Self) -> bool {
        self.data.len() == other.data.len() && self.total_count == other.total_count
    }
}

pub struct DataTableSnapshot {
    pub page: usize,
    pub page_size: usize,
    pub sort_field: Option<String>,
    pub sort_dir: SortDir,
    pub search: String,
}
