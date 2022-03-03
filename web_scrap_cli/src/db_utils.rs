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

pub fn get_selector_records_from_table(conn: &Connection, table: &str)-> Result<Vec<SelectorRecord>, Box<dyn Error>>{
    let sql_request = format!("SELECT (timestamp, url, selector, content) FROM {}", table);
    let mut stmt = conn.prepare(&sql_request)?;
    let record_iter = stmt.query_map([], |row| {
        Ok(SelectorRecord {
            timestamp: row.get(0)?,
            url: row.get(1)?,
            selector: row.get(2)?,
            content: row.get(3)?
        })
    })?;

    let mut records = Vec::new();
    for record in record_iter {
        println!("1 SelectorRecord loaded");
       records.push(record?);
    }
    return Ok(records);
}

pub fn get_row_count(conn: &Connection, table: &str) -> Result<u32, Box<dyn Error>> {
    let sql_request = format!("SELECT COUNT(*) FROM {}", table);

    let mut stmt = conn.prepare(&sql_request)?;
    let rows = stmt.query_map([], |row| row.get(0))?;

    let mut values = Vec::new();
    for val in rows {
       values.push(val?);
    }
    
    println!("Got values : {:?}", values);

    return Ok(values[0]);
}

#[cfg(test)]
mod tests {        
    use std::path::Path; 

    use super::super::*;  // retrieve all from main
    use super::{get_row_count, get_selector_records_from_table };

    #[test]
    fn test_save_selector_records_to_db() {
        let mut conn = Connection::open(Path::new(&"web_scrap_cli_test.db")).unwrap();
        let table = &"selector_record_test";

        match conn.execute(format!("DROP TABLE IF EXISTS {}", table).as_str(), [],) {
            Ok(_updated) => println!("'{}' has been dropped", table),
            Err(err) => panic!("DROP TABLE failed: {}", err),
        }
        
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
        let row_count = get_row_count(&conn, table).unwrap();        
        assert_eq!(row_count, 2, "{:?}", row_count);

        let _rows = get_selector_records_from_table(&conn, table).unwrap();

        match conn.execute(format!("DROP TABLE {}", table).as_str(), [],) {
            Ok(updated) => println!("{} row inserted in '{}'", updated, table),
            Err(err) => panic!("DROP TABLE failed: {}", err),
        }
    }

}