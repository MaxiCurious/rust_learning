use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use csv::{Reader, ReaderBuilder};

use  super::scrap_utils;

pub fn get_file_content(filepath: String) -> Result<String, Box<dyn Error>> {   
    // return the file content as a String
    let full_filepath = Path::new(&filepath);
    println!("Loading file content as string : {}", full_filepath.canonicalize()?.display() );

    let mut file = File::open(full_filepath)?;   
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

pub fn _get_csv_reader(filepath: String, delim: u8) -> Result<Reader<File>, Box<dyn Error>>{
    // return a csv Reader iterator
    let full_filepath = Path::new(&filepath);
    println!("Loading csv file as reader iterator : {}", full_filepath.canonicalize()?.display() );

    let reader = ReaderBuilder::new().delimiter(delim).from_path(full_filepath)?;
    return Ok(reader);
}

pub fn save_records_to_csv<P:AsRef<Path>>(records: &Vec<scrap_utils::SelectorRecord>, outputfilepath: P) -> Result<(), Box<dyn Error>>{
    let mut wtr = csv::Writer::from_path(outputfilepath)?;

    // When writing records with Serde using structs, the header row is written
    // automatically.
    for record in records {
        wtr.serialize(record)?;
    }
    wtr.flush()?;
    Ok(())
}