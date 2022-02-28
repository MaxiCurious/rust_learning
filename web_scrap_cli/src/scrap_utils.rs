use std::time::{SystemTime, UNIX_EPOCH};
use scraper::{Html, Selector};
use reqwest::Client;
use select::document::Document;
use select::predicate::Name;
use serde::{Serialize, Deserialize};
use regex::RegexSet;

use super::file_utils::save_records_to_csv;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SelectorRecord<'a> {
    pub timestamp: u64,
    pub url: &'a str,
    pub selector: &'a str,
    pub content: &'a str,
}

pub async fn get_body_from(client: &Client, url: &str) -> String{
    let response = client.get(url).send().await.unwrap().text().await;  
    return response.expect("Error in the response");     
}

pub async fn print_all_links(content: &str){
    Document::from(content)
        .find(Name("a"))
        .filter_map(|n| n.attr("href"))
        .for_each(|x| println!("{}", x));
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

pub async fn extract_selector_records_to_csv(content: &str, valid_url: &str, selector: &str) -> (){
    
    println!("\nExtracting '{}' CSS Selector items ...", selector);  
    let timestamp: u64 = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Timestamp error !")
                        .as_secs();                                          
    
    // keep only text that match one of following regex 
    let regex_set = RegexSet::new(&[
        r"(?m)^[a-zA-Z]{4,}", // text with at least 4 letters
    ]).unwrap();

    let text_items = get_css_selector_items(content, selector).await;
    let mut records = Vec::new();
    //println!("Current timestamp={}", timestamp.as_secs_f32());
    for item in &text_items{
        if regex_set.is_match(item){
            println!("{}", item);
            records.push(SelectorRecord{timestamp,
                url: valid_url,
                selector: selector,
                content: item});
        }                        
    }
    println!("-------------------\nFound {} items matching regex, for selector '{}' !", records.len(), selector);
    if records.len() > 0 {
        save_records_to_csv(&records, 
            std::env::current_dir().unwrap().join(format!("selector_items_{}.csv", timestamp))
        ).unwrap();
    }     
}

