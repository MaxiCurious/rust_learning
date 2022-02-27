
use std::process;
use std::error::Error;
use reqwest::Client;
use clap::Parser;

use yaml_rust::YamlLoader;

mod config;
use config::{Args, Config};

mod scrap_utils;
use scrap_utils::*;

const YAML_FILEPATH_FROM_HOME: &str = "/work/rust_projects/reqwest_test_config.yaml";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    // use  cli args or use yaml config

    let args = Args::parse();
    let config = Config::new(args.url, args.selector).unwrap_or_else(|err|{
        eprintln!("Problem with Config: {}", err); // eprintln : prints to std error output
        process::exit(1)
    });

    config.print_info();

    let content = config::get_file_content((&YAML_FILEPATH_FROM_HOME).to_string()).expect("Can't get yaml file content !");
    let yaml_vec = YamlLoader::load_from_str(content.as_str())?;   
    let yaml = &yaml_vec[0] ;

    let mut _id = yaml["id"].as_str().map(|s| s.to_string());
    
    run(config).await?;

    return Ok(())
}


pub async fn run(config: Config) -> Result<(), Box<dyn Error>>{    

    let client = Client::builder().cookie_store(true).build()?;
    
    print_all_links(&client, &config.url).await;

    if &config.url.len() == &0usize {
        let text_items = get_css_selector_items(&client, &config.url, &config.selector).await;
        
        for t in text_items{
            println!("{}", t);
        }
    }
    return Ok(());
}
