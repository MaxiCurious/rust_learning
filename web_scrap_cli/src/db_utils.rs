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
                  content         TEXT,
                  host            TEXT NOT NULL
                  )", table_name).as_str(),
        [],
    )?;
    return Ok(());
}

pub fn save_selector_records_to_db(conn: &mut Connection, table_name: &str, records: &Vec<SelectorRecord>) -> Result<(), Box<dyn Error>>{
    
    create_selector_record_table(&conn, table_name)?;
    let transaction = conn.transaction().unwrap(); 
    let stmt_template = format!("INSERT INTO {} (timestamp, url, selector, content, host) VALUES (?1, ?2, ?3, ?4, ?5)", table_name);
    let mut stmt = transaction.prepare_cached(&stmt_template).unwrap();
                            
    for record in records{
        stmt.execute(params![record.timestamp, record.url, record.selector, record.content, record.host]).unwrap();
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
    let sql_request = format!("SELECT timestamp, url, selector, content FROM {}", table);
    let mut stmt = conn.prepare(&sql_request)?;
    let record_iter = stmt.query_map([], |row| {
        Ok(SelectorRecord::new(row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    })?;

    let mut records = Vec::new();
    for record in record_iter {
        println!("1 SelectorRecord loaded");
       records.push(record?);
    }
    return Ok(records);
}

pub fn get_db_table_names(conn: &Connection) -> Result<Vec<String>, Box<dyn Error>>{
    let sql_request = "SELECT name FROM sqlite_schema WHERE type ='table' AND name NOT LIKE 'sqlite_%';";
    let mut stmt = conn.prepare(&sql_request)?;
    let rows = stmt.query_map([], |row| row.get(0))?;

    let mut values = Vec::new();
    for val in rows {
       values.push(val?);
    }
    return Ok(values);
}

pub fn get_row_count(conn: &Connection, table: &str) -> Result<u32, Box<dyn Error>> {
    let sql_request = format!("SELECT COUNT(*) FROM {}", table);

    let mut stmt = conn.prepare(&sql_request)?;
    let rows = stmt.query_map([], |row| row.get(0))?;

    let mut values = Vec::new();
    for val in rows {
       values.push(val?);
    }

    return Ok(values[0]);
}

pub fn get_col_names(conn: &Connection, table: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let sql_request = format!("PRAGMA table_info({});", table);

    let mut stmt = conn.prepare(&sql_request)?;
    let rows = stmt.query_map([], |row| row.get(1))?; // get the second item for the column name

    let mut values = Vec::new();
    for val in rows {
       values.push(val?);
    }
    return Ok(values);
}

pub fn _drop_table(conn: &Connection, table: &str) -> Result<(), Box<dyn Error>>{
    match conn.execute(format!("DROP TABLE {}", table).as_str(), [],) {
        Ok(_updated) => println!("'{}' has been dropped", table),
        Err(err) => panic!("DROP TABLE failed: {}", err),
    }
    return Ok(());
}

pub fn print_db_stats(conn: &Connection) -> Result<(), Box<dyn Error>> {
    println!("--------------\nDatabase Stats:");
    let table_names = get_db_table_names(&conn)?;
    for table in table_names{
        println!(">> '{}' table :", table);
        println!("Columns : {:?}", get_col_names(conn, &table)?);
        println!("{:?} rows", get_row_count(conn, &table)?);  
        let _records = get_selector_records_from_table(conn, &table);
    }
    println!("--------------");
    return Ok(());
}

#[cfg(test)]
mod tests {        
    use std::path::Path; 

    use super::super::*;  // retrieve all from main
    use super::get_row_count;

    #[test]
    fn test_save_selector_records_to_db() {
        let mut conn = Connection::open(Path::new(&"web_scrap_cli_test.db")).unwrap();
        let table = &"selector_record_test";

        match conn.execute(format!("DROP TABLE IF EXISTS {}", table).as_str(), [],) {
            Ok(_updated) => println!("'{}' has been dropped", table),
            Err(err) => panic!("DROP TABLE failed: {}", err),
        }
        
        let mut records: Vec<SelectorRecord> = Vec::new();
        records.push(SelectorRecord::new(get_timestamp_now(), 
                                    String::from(r"http:\\www.test.fr"),
                                    String::from("a"),
                                    String::from("blablabla")
                                ));
        records.push(SelectorRecord::new(get_timestamp_now(), 
                                String::from(r"http:\\www.test-other.fr"),
                                String::from("p"),
                                String::from("bliblibli")
                            ));

        save_selector_records_to_db(&mut conn, table, &records).unwrap();
        let row_count = get_row_count(&conn, table).unwrap();        
        assert_eq!(row_count, 2, "{:?}", row_count);

        let _result = print_db_stats(&conn);
        _drop_table(&conn, table).unwrap();
    }

}