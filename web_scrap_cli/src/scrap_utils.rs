
use scraper::{Html, Selector};
use reqwest::Client;
use select::document::Document;
use select::predicate::Name;

async fn get_body_from(client: &Client, url: &str) -> String{
    let response = client.get(url).send().await.unwrap().text().await;  
    return response.expect("");     
}

pub async fn print_all_links(client: &Client, url: &str, ){
    Document::from(get_body_from(client, url).await.as_str())
    .find(Name("a"))
    .filter_map(|n| n.attr("href"))
    .for_each(|x| println!("{}", x));
}

pub async fn get_css_selector_items(client: &Client, url: &str,  selector: &str) -> Vec<String>{
    let body_html = Html::parse_document(get_body_from(client, url).await.as_str());
    let selector = Selector::parse(selector).unwrap();

    let mut results: Vec<String> = Vec::new();

    for item in body_html.select(&selector){
        results.push(item.text().map(|s| s.to_string()).collect());
    }
   return results;
    
}

