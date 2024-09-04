mod file_structure;

use file_structure::{FitFile, FitHeader};
use std::fs::File;
use std::io::{self, Read};

fn read_file_to_vector(file_path: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?;

    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer)?;

    Ok(buffer)
}

fn dump_file_content(file_data: &[u8], start: Option<usize>, end: Option<usize>) {
    let file_len = file_data.len();
    let start_index = start.unwrap_or(0).min(file_len);
    let end_index = end.unwrap_or(file_data.len()).min(file_len);

    for &byte in &file_data[start_index..end_index] {
        print!("{:02x}", byte);
    }
    println!();
}

fn main() -> io::Result<()> {
    let file_path = "Evening_Ride.fit";

    //let data = read_file_to_vector(file_path)?;

    //println!("File read successfully, {} bytes read.", data.len());

    //println!("Dumping the Header:");

    //dump_file_content(&data, Some(0), Some(100));

    let mut file = File::open(file_path)?;

    let header = FitHeader::from_reader(&mut file)?;

    println!("{:?}", header);

    header.pretty_print();
    Ok(())
}
