mod file_structure;

use file_structure::{FitFile, FitHeader};
use rfd::FileDialog;
use std::fs::File;
use std::io::{self, Read};
use std::process;

use crate::file_structure::{FitRecord, RecordHeader};

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
    //let file_path = "Evening_Ride.fit";

    let file_path = match FileDialog::new()
        .set_title("Select a .fit File")
        .set_directory(".")
        .pick_file()
    {
        Some(p) => p,
        None => {
            println!("No file selected");
            process::exit(1);
        }
    };

    let mut file = File::open(&file_path)?;

    let header = FitHeader::from_reader(&mut file)?;

    println!("{:?}", header);
    println!("{:02x}", header.data_type);

    //header.pretty_print();

    //let nh = RecordHeader::new(0b0100_0000);
    //println!("{:?}", nh);

    //let th = RecordHeader::new(0b1100_0010);
    //println!("{:?}", th);

    //let fr = FitRecord::new(&0b0100_0000);
    //println!("{:?}", fr);

    //let frd = FitRecord::new(&0b1100_0010);
    //println!("{:?}", frd);
    let file_path_str = file_path.to_string_lossy();
    let mut fit_file = FitFile::new(&file_path_str)?;

    Ok(())
}
