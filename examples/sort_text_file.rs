use std::path::{Path, PathBuf};
use anyhow::Error;
use text_file_sort::field::Field;
use text_file_sort::field_type::FieldType;
use text_file_sort::order::Order;
use text_file_sort::sort::Sort;

use tikv_jemallocator::Jemalloc;
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn sort_random(input_path: &Path, output_path: &Path) -> Result<(), Error> {
    let mut text_file = Sort::new(vec![input_path.to_path_buf()], output_path.to_path_buf());
    text_file.with_fields(vec![Field::new(0, FieldType::String).with_random(true)]);
    text_file.sort()?;
    Ok(())
}

fn sort_lines_ascending(input_path: &Path, output_path: &Path) -> Result<(), Error> {
    // ascending order is the default
    let text_file = Sort::new(vec![input_path.to_path_buf()], output_path.to_path_buf());
    text_file.sort()?;
    Ok(())
}

fn sort_lines_descending(input_path: &Path, output_path: &Path) -> Result<(), Error> {
    let mut text_file = Sort::new(vec![input_path.to_path_buf()], output_path.to_path_buf());
    text_file.with_order(Order::Desc);
    text_file.sort()?;
    Ok(())
}

fn sort_records(input_path: &Path, output_path: &Path) -> Result<(), Error> {
    let mut text_file = Sort::new(vec![input_path.to_path_buf()], output_path.to_path_buf());
    text_file.add_field(Field::new(3, FieldType::String));
    text_file.add_field(Field::new(2, FieldType::Integer));
    text_file.sort()?;
    Ok(())
}


// cargo run -r --example sort_text_file
pub fn main() -> Result<(), Error> {
    let input_path = PathBuf::from("./tests/fixtures/sorted-1000.dat");
    let random_path = PathBuf::from("./target/random-1000.dat");
    let ascending_path = PathBuf::from("./target/ascending-1000.dat");
    let descending_path = PathBuf::from("./target/descending-1000.dat");
    let records_path = PathBuf::from("./target/records-1000.dat");

    sort_random(&input_path, &random_path)?;
    sort_lines_ascending(&random_path, &ascending_path)?;
    sort_lines_descending(&random_path, &descending_path)?;
    sort_records(&random_path, &records_path)?;

    Ok(())
}