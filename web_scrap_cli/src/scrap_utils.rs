use std::time::{SystemTime, UNIX_EPOCH};
use std::error::Error;
use scraper::{Html, Selector};
use reqwest::Client;
use select::document::Document;
use select::predicate::Name;
use serde::{Serialize, Deserialize};
use regex::RegexSet;
use rusqlite::{params, Connection, Result};

use super::file_utils::save_records_to_csv;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SelectorRecord<'a> {
    pub timestamp: u64,
    pub url: &'a str,
    pub selector: &'a str,
    pub content: &'a str,
}

pub async fn get_body_from(client: &Client, url: &str) -> String{
    let response = client.get(url).send().await.unwrap().text().await;  
    return response.expect("Error in the response");     
}

pub async fn print_all_links(content: &str){
    Document::from(content)
        .find(Name("a"))
        .filter_map(|n| n.attr("href"))
        .for_each(|x| println!("{}", x));
}

pub async fn get_css_selector_items(content: &str,  selector: &str) -> Vec<String>{
    let body_html = Html::parse_document(content);
    let selector = Selector::parse(selector).unwrap();

    let mut results: Vec<String> = Vec::new();

    for item in body_html.select(&selector){
        results.push(item.text().map(|s| s.to_string()).collect());
    }    
   return results;
    
}

fn get_timestamp_now() -> u64{
    return SystemTime::now().duration_since(UNIX_EPOCH)
                        .expect("Timestamp error !")
                        .as_secs(); 
}

pub async fn extract_selector_records(content: &str, valid_url: &str, selector: &str, 
                                    save_to_csv: bool, db_conn: &Option<Connection>, 
                                    table_name: &str) -> Result<(), Box<dyn Error>>{
    
    println!("\nExtracting '{}' CSS Selector items ...", selector);  
    let timestamp = get_timestamp_now();                                          
    
    // keep only text that match one of following regex 
    let regex_set = RegexSet::new(&[
        r"(?m)^[a-zA-Z]{4,}", // text with at least 4 letters
    ])?;

    let text_items = get_css_selector_items(content, selector).await;
    let mut records = Vec::new();
    //println!("Current timestamp={}", timestamp.as_secs_f32());
    for item in &text_items{
        if regex_set.is_match(item){
            println!("{}", item);
            records.push(SelectorRecord{timestamp,
                url: valid_url,
                selector: selector,
                content: item});
        }                        
    }
    println!("-------------------\nFound {} items matching regex, for selector '{}' !", records.len(), selector);
    if records.len() > 0 {
        if save_to_csv {
            save_records_to_csv(&records, 
                std::env::current_dir()?.join(format!("{}{}.csv", super::CSV_NAME_PREFIX, timestamp))
            )?;
        };
        match db_conn {
            Some(conn) => save_selector_records_to_db(&conn, table_name, &records)?,
            _ => {}           
        };        
    }  
    return Ok(());
}

fn create_selector_record_table(conn: &Connection, table_name: &str) -> Result<(), Box<dyn Error>> {
    conn.execute(
        format!("CREATE TABLE IF NOT EXISTS {} (
                  id              INTEGER PRIMARY KEY,
                  timestamp       TIMESTAMP,
                  url             TEXT NOT NULL,
                  selector        TEXT NOT NULL,
                  content         TEXT
                  )", table_name).as_str(),
        [],
    )?;
    return Ok(());
}

pub fn save_selector_records_to_db(conn: &Connection, table_name: &str, records: &Vec<SelectorRecord<'_>>) -> Result<(), Box<dyn Error>>{
    create_selector_record_table(&conn, table_name)?;
    for record in records{
        match &conn.execute(format!("INSERT INTO {} (timestamp, url, selector, content) VALUES (?1, ?2, ?3, ?4)", table_name).as_str(),
                            params![record.timestamp, record.url, record.selector, record.content]) 
        {
            Ok(updated) => println!("{} row inserted", updated),
            Err(err) => println!("update failed: {}", err),
        }
    }    
    return Ok(());
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_save_selector_records_to_db() {
        let conn = Connection::open(Path::new(&"./test_temp/web_scrap_cli_test.db")).unwrap();
        let table = &"selector_record_test";
        let mut records: Vec<SelectorRecord> = Vec::new();

        records.push(SelectorRecord{timestamp: get_timestamp_now(),
                                    url: r"http:\\www.test.fr",
                                    selector: "a",
                                    content: "blablabla"}
                    );
        records.push(SelectorRecord{timestamp: get_timestamp_now(),
            url: r"http:\\www.test.fr",
            selector: "p",
            content: "blibli"}
        );

        save_selector_records_to_db(&conn, table, &records).unwrap();
        match conn.execute(format!("DROP TABLE {}", table).as_str(), [],) {
            Ok(updated) => println!("{} row inserted in '{}'", updated, table),
            Err(err) => panic!("DROP TABLE failed: {}", err),
        }
    }

}