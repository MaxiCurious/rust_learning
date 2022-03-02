use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use csv::{Reader, ReaderBuilder};

use  super::scrap_utils::SelectorRecord;

pub fn get_timestamp_now() -> u64{
    return SystemTime::now().duration_since(UNIX_EPOCH)
                        .expect("Timestamp error !")
                        .as_secs(); 
}

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

pub fn _get_selector_records_from_csv(filepath: String, delim: u8) -> Result<(), Box<dyn Error>>{
    let mut reader = _get_csv_reader(filepath, delim)?;
    let mut raw_record = csv::ByteRecord::new();
    let headers = reader.byte_headers()?.clone();
    let mut _records: Vec<SelectorRecord> = Vec::new();

    while reader.read_byte_record(&mut raw_record)? {
        let _record: SelectorRecord = raw_record.deserialize(Some(&headers))?;
        //records.push(record.clone());        
    }
    Ok(())
}

pub async fn save_records_to_csv<P:AsRef<Path>>(records: &Vec<SelectorRecord>, outputfilepath: P) -> Result<(), Box<dyn Error>>{
    let mut wtr = csv::Writer::from_path(outputfilepath)?;

    // When writing records with Serde using structs, the header row is written
    // automatically.
    for record in records {
        wtr.serialize(record)?;
    }
    wtr.flush()?;
    Ok(())
}