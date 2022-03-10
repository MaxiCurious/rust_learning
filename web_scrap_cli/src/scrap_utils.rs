use std::error::Error;
use scraper::{Html, Selector};
use reqwest::Client;
use select::document::Document;
use select::predicate::Name;
use serde::{Serialize, Deserialize};
use regex::{RegexSet, Regex};
use url::Url;

use  super::file_utils::get_timestamp_now;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SelectorRecord {
    pub timestamp: u64,
    pub url: String,
    pub selector: String,
    pub content: String,
    pub host: String
}

impl SelectorRecord{
    pub fn new(timestamp: u64, url: String, selector: String, content: String) -> SelectorRecord{
        let host:String = get_host_from_url(&url).unwrap();
        return SelectorRecord{timestamp, url, selector, content, host}
    }
}

pub async fn get_body_from(client: &Client, url: &str) -> String{
    let response = client.get(url).send().await.unwrap().text().await;  
    return response.expect("Error in the response");     
}

pub async fn extract_all_links(content: &str) -> Vec<String>{
    println!("Links in the page :\n");
    let links: Vec<String> = Document::from(content).find(Name("a"))                                  
                                  .filter_map(|n| n.attr("href"))
                                  .map(|s| String::from(s))
                                  .collect::<Vec<String>>();
    return links;
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

pub fn get_host_from_url(url: &str) -> Result<String, Box<dyn Error>> {
    let url = Url::parse(url)?;
    let host: String = format!("{}", url.host().unwrap());
    //assert_eq!(url.scheme(), "ftp");
    //assert_eq!(url.host(), Some(Host::Domain("rust-lang.org")));
    //assert_eq!(url.port_or_known_default(), Some(21));
    Ok(host)
}

pub async fn extract_selector_records(content: &str, valid_url: &str, selector: &str) -> Result<Vec<SelectorRecord>, Box<dyn Error>>{    
    println!("\nExtracting '{}' CSS Selector items ...", selector);  
    let timestamp = get_timestamp_now();                                          
    
    // keep only text that match one of following regex 
    let regex_set = RegexSet::new(&[
        r"(?m)^[a-zA-Z]{4,}", // text with at least 4 letters
        //r"(\w.+\s).+" // text with at least 2 words
    ])?;
    
    let re = Regex::new(r"\s\s+").unwrap(); // to find multiple spaces and remove them

    let text_items = get_css_selector_items(content, selector).await;
    let mut records = Vec::new();

    //println!("Current timestamp={}", timestamp.as_secs_f32());
    for item in &text_items{
        if regex_set.is_match(item){
            let cleaned_item = re.replace_all(item, " ").to_string();
            println!("{}", cleaned_item);
            records.push(SelectorRecord::new(timestamp, String::from(valid_url), String::from(selector), cleaned_item));
        }                        
    }
    println!("-------------------\nFound {} items matching regex, for selector '{}' !", records.len(), selector);
    
    return Ok(records);
}
