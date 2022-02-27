use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};
use reqwest::Client;
use clap::Parser;

mod config;
use config::{Args, Config};

mod scrap_utils;
use scrap_utils::*;

mod file_utils;
use file_utils::save_records_to_csv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{    
    // use  cli args with possible yaml config use

    let args = Args::parse();
    let config = args.build_config();

    config.print_info();    
    //run(config).await?;
    if let Err(error) = run(config).await {
        println!("Run error: {:?}", error)        
    }

    return Ok(())
}


pub async fn run(config: Config) -> Result<(), Box<dyn Error>>{    

    let client = Client::builder().cookie_store(true).build()?;
    
    match &config.url{
        Some(valid_url) => {
            print_all_links(&client, valid_url).await;
            
            match &config.selector {
                Some(v) => {
                    println!("\nExtracting '{}' CSS Selector items ...", v);
                    let text_items = get_css_selector_items(&client, valid_url, v).await;
                    let mut records = Vec::new();
                    let timestamp = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .expect("Timestamp error !");
                    //println!("Current timestamp={}", timestamp.as_secs_f32());
                    for item in &text_items{
                        println!("{}", item);
                        records.push(SelectorRecord{timestamp: timestamp.as_secs(),
                                                    selector: v,
                                                    content: item});
                    }
                    save_records_to_csv(&records, 
                        std::env::current_dir()?.join(format!("selector_items_{}.csv", timestamp.as_secs()))
                    )?;
                },
                _ => println!("No CSS Selector used")
            }   
        
        },
        _ => {
            println!("Target URL is not defined ...");
        }
    };
    
    return Ok(());
}
