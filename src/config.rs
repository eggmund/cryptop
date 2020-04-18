use yaml_rust::Yaml;
use std::path::Path;
use std::fmt::Display;

#[derive(Debug)]
pub struct Config {
    symbol: String,
}

impl Config {
    pub fn from_path<P: AsRef<Path> + Display>(path: P) -> Config {
        use yaml_rust::YamlLoader;
        use std::fs;
        use std::io::Read;
    
        let mut conf_file_string = String::new();
        fs::File::open(path)
            .expect(&format!("Could not find config file."))
            .read_to_string(&mut conf_file_string)
            .expect("Could not read config file to string.");
        
        let conf_yaml = YamlLoader::load_from_str(&conf_file_string).unwrap();
        if conf_yaml.is_empty() { panic!("Error: Config file is empty...") }
        
        Config::from(&conf_yaml[0])
    }
}

impl From<&Yaml> for Config {
    fn from(conf: &Yaml) -> Config {
        Config {
            symbol: conf["symbol"].as_str()
                .expect("Could not parse 'symbol' field as string in config file.").into(),
        }
    }
}