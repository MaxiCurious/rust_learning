use std::error::Error;

use rusqlite::{params, Connection, Result};

use super::scrap_utils::SelectorRecord;

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

pub fn save_selector_records_to_db(conn: &mut Connection, table_name: &str, records: &Vec<SelectorRecord>) -> Result<(), Box<dyn Error>>{
    
    create_selector_record_table(&conn, table_name)?;
    let transaction = conn.transaction().unwrap(); 
    let stmt_template = format!("INSERT INTO {} (timestamp, url, selector, content) VALUES (?1, ?2, ?3, ?4)", table_name);
    let mut stmt = transaction.prepare_cached(&stmt_template).unwrap();
                            
    for record in records{
        stmt.execute(params![record.timestamp, record.url, record.selector, record.content]).unwrap();
    //     match &conn.execute(format!("INSERT INTO {} (timestamp, url, selector, content) VALUES (?1, ?2, ?3, ?4)", table_name).as_str(),
    //                         params![record.timestamp, record.url, record.selector, record.content]) 
    //     {
    //         Ok(updated) => println!("{} row inserted", updated),
    //         Err(err) => println!("update failed: {}", err),
    //     }
    }
    drop(stmt); // Added to release transaction ownership and solve the followng: borrow might be used here, when `stmt` is dropped and runs the `Drop` code for type `rusqlite::CachedStatement`
    transaction.commit().unwrap();
    
    return Ok(());
}


#[cfg(test)]
mod tests {    
    use super::super::*;  // retrieve all from main
    use std::path::Path;    

    #[test]
    fn test_save_selector_records_to_db() {
        let mut conn = Connection::open(Path::new(&"web_scrap_cli_test.db")).unwrap();
        let table = &"selector_record_test";
        let mut records: Vec<SelectorRecord> = Vec::new();

        records.push(SelectorRecord{timestamp: get_timestamp_now(),
                                    url: String::from(r"http:\\www.test.fr"),
                                    selector: String::from("a"),
                                    content: String::from("blablabla")}
                    );
        records.push(SelectorRecord{timestamp: get_timestamp_now(),
            url: String::from(r"http:\\www.test.fr"),
            selector: String::from("p"),
            content: String::from("blibli")}
        );

        save_selector_records_to_db(&mut conn, table, &records).unwrap();
        match conn.execute(format!("DROP TABLE {}", table).as_str(), [],) {
            Ok(updated) => println!("{} row inserted in '{}'", updated, table),
            Err(err) => panic!("DROP TABLE failed: {}", err),
        }
    }

}