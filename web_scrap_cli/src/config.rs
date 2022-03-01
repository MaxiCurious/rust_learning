use std::env;
use clap::Parser;
use std::error::Error;
use yaml_rust::YamlLoader;

use  super::file_utils;


#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(short, long, help="Defines the Url to scrap")]
    pub url: Option<String>,

    #[clap(short, long, help="Defines the CSS Selector to filter")]
    pub selector: Option<String>,

    #[clap(short, long, help="Defines the yaml file to use as config for URL and SELECTOR")]
    pub yaml_cfg: Option<String>,

    #[clap(short, long, help="Save result to given database")]
    pub db: Option<String>,

    #[clap(short, long, default_value="selector_record", help="Save result to given table, default to 'selector_record'")]
    pub table: String,

    #[clap(long, help="Save results to a csv file")]
    pub to_csv: bool,    
}

impl Args{
    pub fn build_config(&self) -> Config{
        let config: Config = match &self.yaml_cfg {
            Some(v) => Config::new_from_yaml_file(v, &self.to_csv, &self.db).unwrap(),
            _ => {
                let mut url_selector_vec = Vec::new();
                url_selector_vec.push(
                    UrlSelectorPair{url: self.url.as_ref().unwrap().to_string(),
                                    selector: self.selector.as_ref().unwrap().to_string()}
                );
                Config::new(&url_selector_vec, &self.to_csv, &self.db, &self.table)
            }
        };
        return config;
    }
}

#[derive(Debug, Clone)]
pub struct UrlSelectorPair{
    pub url: String,
    pub selector: String
}

#[derive(Debug)]
pub struct Config{
    pub url_selectors: Vec<UrlSelectorPair>,
    pub save_to_csv: bool,
    pub db_path: Option<String>,
    pub table: String,
    pub env_arg1: bool
}

impl Config{
    pub fn new(url_selectors: &Vec<UrlSelectorPair>, save_to_csv:&bool, db_path: &Option<String>, table: &String) -> Config{   
        let env_arg1 = env::var("WEB_SCRAP_CLI_ARG1").is_err();        
        return Config {url_selectors: url_selectors.clone(), 
            save_to_csv: save_to_csv.clone(),
            db_path: db_path.clone(),
            table: table.clone(),
            env_arg1};
    }

    pub fn new_from_yaml_file(yaml_cfg: &String, save_to_csv:&bool, db_path: &Option<String>) -> Result<Config, Box<dyn Error>>{
        println!("Current dir : {:?}", std::env::current_dir());
        println!("Current exe: {:?}", std::env::current_exe());

        let content = file_utils::get_file_content((&yaml_cfg).to_string()).expect("Can't get yaml file content !");
        return Config::new_from_yaml_string(&content, save_to_csv, db_path);
    }

    pub fn new_from_yaml_string(yaml_content: &str, save_to_csv:&bool, db_path: &Option<String>)-> Result<Config, Box<dyn Error>>{
        let yaml_vec = YamlLoader::load_from_str(yaml_content)?;   
        let yaml = &yaml_vec[0] ;

        let url_selector_tuples = yaml["url_selector_tuples"].as_vec().expect("Couldn't find 'url_selevtor_tuples' list in the yaml !");
        let mut url_selectors: Vec<UrlSelectorPair> = Vec::new();
        let mut yaml_item;
        for elem in url_selector_tuples {
            yaml_item = elem.as_vec().unwrap();
            url_selectors.push(
                UrlSelectorPair{url: yaml_item[0].as_str().unwrap().to_string(),
                                selector: yaml_item[1].as_str().unwrap().to_string()})
        }
        return Ok(Config::new(&url_selectors, save_to_csv, db_path, &String::from("selector_record")));
    }

    pub fn print_info(&self){
        println!("{:?}", &self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_from_yaml_ok() {
        let fake_yaml_content: &str = r#"
        url_selector_tuples: 
            - [https://www.google.fr, div]
        "#;
        let config = Config::new_from_yaml_string(&fake_yaml_content, &false, &None);
        assert!(config.is_ok(), "{}", format!("config = {:#?}", config));  

        let config_ok = &config.unwrap();
        let url_selector:&UrlSelectorPair = &config_ok.url_selectors[0];
        assert_eq!(url_selector.url, "https://www.google.fr".to_string());
        assert_eq!(url_selector.selector, "div".to_string());          
        
    }

    #[test]
    #[should_panic]
    fn test_new_from_yaml_wrong_list_panic() {
        let fake_yaml_content: &str = r#"
        wrong_list_name: 
            - [https://www.google.fr, div]
        "#;
        _ = Config::new_from_yaml_string(&fake_yaml_content, &false, &None);
    }
    #[test]
    #[should_panic]
    fn test_new_from_yaml_missing_item_panic() {
        let fake_yaml_content: &str = r#"
        url_selector_tuples: 
            - [https://www.google.fr]
        "#;
        _ = Config::new_from_yaml_string(&fake_yaml_content, &false, &None);
    }

}