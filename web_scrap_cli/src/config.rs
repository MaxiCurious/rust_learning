use std::env;
use clap::Parser;
use std::error::Error;
use std::fs::File;
use std::io::Read;

pub fn get_file_content(filepath: String) -> Result<String, Box<dyn Error>> {
    let home = std::env::var("HOME")?;
    let full_filepath = format!("{}{}", home, &filepath);
    println!("Loading credential info from file: {}", full_filepath );

    let mut config_file = File::open(full_filepath)?;   
    let mut content = String::new();
    config_file.read_to_string(&mut content)?;
    Ok(content)
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(short, long)]
    pub url: String,

    #[clap(short, long, default_value = "")]
    pub selector: String    
}

pub struct Config{
    pub url: String,
    pub selector: String,
    pub env_arg1: bool,
}

impl Config{
    pub fn new(url: String, selector: String) -> Result<Config, String>{   
        let env_arg1 = env::var("WEB_SCRAP_CLI_ARG1").is_err();        
        return Ok(Config {url, selector, env_arg1});
    }

    pub fn print_info(&self){
        println!("Config.url = '{}'", &self.url);
        println!("Config.selector = '{}'", &self.selector);
        println!("Config.env_arg1 = '{}'", &self.env_arg1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_config_ok_test() {

        let config = Config::new("path".to_string(), "bla".to_string());
        assert!(config.is_ok())
    }

}