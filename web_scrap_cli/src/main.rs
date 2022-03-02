use std::time::Instant;
use std::io::Write;
use reqwest::Client;
use clap::Parser;
use rusqlite::Connection;
//use futures::prelude::*;
use futures::future::join_all;
use tokio::task;
use log::*;

mod config;
use config::{Args, Config, UrlSelectorPair};

mod scrap_utils;
use scrap_utils::*;

mod file_utils;
use file_utils::{save_records_to_csv, get_timestamp_now};

mod db_utils;
use db_utils::save_selector_records_to_db;

// Not in parallel : Total duration = 1.9254413 sec.

const CSV_NAME_PREFIX: &str = "selector_items_";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;


fn main() -> Result<()>{    
    
    let start = std::time::Instant::now();

    // setup the format for logs
    env_logger::Builder::from_default_env().format(move |buf, rec| {
        let t = start.elapsed().as_secs_f32();
        writeln!(buf, "{:.03} [{}] - {}", t, rec.level(), rec.args())
    }).init();

    // use cli args with possible yaml config use   
    let args = Args::parse();
    let config = args.build_config();
    config.print_info();    
    println!("-------------------\nDuration to build config is: {:?}\n", start.elapsed());
      
    // Start asynced work
    let rt = tokio::runtime::Runtime::new().unwrap();    
    match rt.block_on(run(config)) {
        Ok(_) => info!("Done"),
        Err(e) => error!("Run() : An error ocurred: {}", e),
    };

    println!("\n_________\nTotal duration = {} sec.\nTerminating program...", start.elapsed().as_secs_f32());
    return Ok(())
}

pub async fn run(config: Config) -> Result<()>{    
    
    let client = Client::builder().cookie_store(true).build()?;

    // separate threads for parrallism
    let mut futures = vec![];    
    for url_selector in &config.url_selectors {
        let fut = task::spawn(handle_request(client.clone(), url_selector.clone())
        );
        futures.push(fut);               
    }

    // Retrieve all selector items 
    let results = join_all(futures).await;

    // Setup db connection if required
    let mut conn: Option<Connection> = match &config.db_path {
        Some(p) => {
            let new_conn: Connection = Connection::open(&p).unwrap();
            new_conn.execute_batch("PRAGMA journal_mode = OFF;
                                    PRAGMA synchronous = 0;
                                    PRAGMA cache_size = 1000000;
                                    PRAGMA locking_mode = EXCLUSIVE;
                                    PRAGMA temp_store = MEMORY;",
                                    ).expect("db connection PRAGMA error !");
            Some(new_conn)
        },
        _ => None
    }; 

    let mut i: usize = 0;
    for result in results{
        let records = result?.unwrap();
        if records.len() > 0 {
            let req_id = format!("{}_{}", get_timestamp_now(), i);
            handle_records(&records, &mut conn, config.table.clone(), config.save_to_csv.clone(), req_id).await.unwrap();
        }        
        i += 1;
    }
    return Ok(());
}

pub async fn handle_request(client: Client, url_selector: UrlSelectorPair) -> Result<Vec<SelectorRecord>> {
            
    println!("Sending request ...");
    let start = Instant::now(); 
    let content = get_body_from(&client, &url_selector.url).await;  
    println!("Received request content !");
    print_all_links(&content).await;    
    let records = extract_selector_records(&content, &url_selector.url, &url_selector.selector).await.unwrap();
    println!("-------------------\nDuration to handle request is: {:?}\n", start.elapsed());
      
    return Ok(records);
}


pub async fn handle_records(records: &Vec<SelectorRecord>, conn: &mut Option<Connection>, table: String, save_to_csv: bool, req_id: String) -> Result<()> {
    let start = Instant::now();         
    
    if save_to_csv {
        save_records_to_csv(&records, 
            std::env::current_dir()?.join(format!("{}{}.csv", CSV_NAME_PREFIX, req_id))
        ).await.unwrap();
    };
    match conn {
        Some(valid_conn) => save_selector_records_to_db(valid_conn, &table, &records).unwrap(),        
        _ => {}           
    };        
      
    println!("__________________\nDuration to handle records is: {:?}\n", start.elapsed());  
    return Ok(());
}