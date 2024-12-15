use tabled::{
    settings::{Alignment, Style},
    Table, Tabled,
};

pub fn table<T: Tabled>(tabled: &Vec<T>) -> String {
    Table::new(tabled)
        .with(Style::modern_rounded())
        .with(Alignment::center())
        .with(Alignment::center_vertical())
        .to_string()
}
