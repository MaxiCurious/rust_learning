use std::error::Error;
use std::time::Instant;
use reqwest::Client;
use clap::Parser;


mod config;
use config::{Args, Config};

mod scrap_utils;
use scrap_utils::*;

mod file_utils;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{    
    // use  cli args with possible yaml config use   

    let args = Args::parse();
    let config = args.build_config();
    config.print_info();    

    if let Err(error) = run(config).await {
        println!("Run error: {:?}", error)        
    }

    return Ok(())
}

pub async fn run(config: Config) -> Result<(), Box<dyn Error>>{    
    // Execute the config 
    match &config.url{
        Some(valid_url) => {
            let start = Instant::now(); 
            let mut duration;
            let client = Client::builder().cookie_store(true).build()?;
            let content = get_body_from(&client, valid_url).await;
            duration = start.elapsed();
            println!("-------------------\nTime elapsed is: {:?}\n", duration);

            print_all_links(content.as_str()).await;
            duration = start.elapsed();
            println!("-------------------\nTime elapsed is: {:?}\n", duration);

            match &config.selector {
                Some(v) => extract_selector_records_to_csv(content.as_str(), valid_url, v).await,
                _ => println!("No CSS Selector used")
            } 
            duration = start.elapsed();
            println!("__________________\nTotal time elapsed is: {:?}\n", duration);
        },
        _ => {
            println!("Target URL is not defined ...");
        }
    };
    
    return Ok(());
}
