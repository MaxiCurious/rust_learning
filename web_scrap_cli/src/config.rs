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

    #[clap(long, help="Save results to a csv ")]
    pub to_csv: bool,    
}

impl Args{
    pub fn build_config(&self) -> Config{
        let config: Config = match &self.yaml_cfg {
            Some(v) => Config::new_from_yaml_file(v, &self.to_csv, &self.db).unwrap(),
            _ => Config::new(&self.url, &self.selector, &self.to_csv, &self.db, &self.table)
        };
        return config;
    }
}

#[derive(Debug)]
pub struct Config{
    pub url: Option<String>,
    pub selector: Option<String>,
    pub save_to_csv: bool,
    pub db_path: Option<String>,
    pub table: String,
    pub env_arg1: bool,
}

impl Config{
    pub fn new(url: &Option<String>, selector: &Option<String>, save_to_csv:&bool, db_path: &Option<String>, table: &String) -> Config{   
        let env_arg1 = env::var("WEB_SCRAP_CLI_ARG1").is_err();        
        return Config {url: url.clone(), 
            selector: selector.clone(), 
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

        let url = yaml["url"].as_str().map(|s| s.to_string());
        let selector = yaml["selector"].as_str().map(|s| s.to_string());
        return Ok(Config::new(&url, &selector, save_to_csv, db_path, &String::from("selector_record")));
    }

    pub fn print_info(&self){
        match &self.url {
            Some(v) => println!("Config.url = '{}'", v),
            _ => println!("Config.url = None")
        } 
        match &self.selector {
            Some(v) => println!("Config.selector = '{}'", v),
            _ => println!("Config.selector = None")
        }  
        println!("Config.env_arg1 = '{}'", &self.env_arg1);             
        
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_from_yaml_ok() {
        let fake_yaml_content: &str = r#"
        url: "https://www.google.fr"
        selector: "div"
        "#;
        let config = Config::new_from_yaml_string(&fake_yaml_content, &false, &None);
        assert!(config.is_ok(), "{}", format!("config = {:#?}", config));  

        let config_ok = &config.unwrap();
        match &config_ok.url {
            Some(v) => assert_eq!(v, &"https://www.google.fr".to_string()),
            _ => panic!("config_ok.url shouldn't be None !")
        }
        
        match &config_ok.selector {
            Some(v) => assert_eq!(v, &"div".to_string()),
            _ => ()
        }        
        
    }

}