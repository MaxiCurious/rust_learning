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

    // Start async tasks
    let rt = tokio::runtime::Runtime::new().unwrap();    
    match rt.block_on(run(config)) {
        Ok(_) => info!("Done"),
        Err(e) => error!("Run() : An error ocurred: {}", e),
    };

    println!("\n_________\nTotal duration = {} sec.\nTerminating program...", start.elapsed().as_secs_f32());
    return Ok(())
}

pub async fn run(config: Config) -> Result<()>{    
    
    let mut futures = vec![];
    for url_selector in &config.url_selectors {
        let fut = task::spawn(handle_request(url_selector.clone())
        );
        futures.push(fut);               
    }
    let results = join_all(futures).await;

    // Extract selector items and save in csv or db
    let conn: Option<Connection>; 
    match &config.db_path {
        Some(p) => conn = Some(Connection::open(&p)?),
        _ => conn = None
    }; 
    let mut i: usize = 0;
    for result in results{
        let valid_res:String = result?.unwrap();
        handle_content(&valid_res,
            &config.url_selectors[i],
            &conn,
            config.table.clone(),
            config.save_to_csv.clone()).await.unwrap();
        i += 1;
    }
    return Ok(());
}

pub async fn handle_request(url_selector: UrlSelectorPair) -> Result<String> {
            
    let start = Instant::now(); 
    let client = Client::builder().cookie_store(true).build()?;
    let content = get_body_from(&client, &url_selector.url).await;

    print_all_links(content.as_str()).await;
    println!("-------------------\nDuration to handle request is: {:?}\n", start.elapsed());
      
    return Ok(content);
}

pub async fn handle_content(content: &str, url_selector: &UrlSelectorPair, conn: &Option<Connection>, table: String, save_to_csv: bool) -> Result<()> {
    let start = Instant::now();     
    extract_selector_records(content, 
                            &url_selector.url, 
                            &url_selector.selector, 
                            save_to_csv, 
                            conn,
                            &table
                            ).await.unwrap();
    println!("__________________\nDuration to handle content is: {:?}\n", start.elapsed());  
    return Ok(());

}
