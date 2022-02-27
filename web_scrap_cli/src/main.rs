use std::error::Error;
use reqwest::Client;
use clap::Parser;

mod config;
use config::{Args, Config};

mod scrap_utils;
use scrap_utils::*;

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
                    let text_items = get_css_selector_items(&client, valid_url, v).await;
                    for t in text_items{
                        println!("{}", t);
                    }
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
