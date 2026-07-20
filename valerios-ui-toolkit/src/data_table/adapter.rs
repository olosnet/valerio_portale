use crate::data_table::types::{DataTableResponse, DataTableSnapshot};

pub trait DataTableAdapter<T: Clone + PartialEq + 'static> {
    fn fetch(&self, state: DataTableSnapshot) -> DataTableResponse<T>;
}
