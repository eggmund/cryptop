mod config;

use binance::api::*;
use binance::market::*;

use std::thread;
use std::time::Duration;
use std::path::Path;
use std::fmt::Display;

use config::Config;

const CONFIG_PATH_STR: &'static str = "./config.yaml";
const PRICE_QUERY_SLEEP_TIME: Duration = Duration::from_secs(1);       // 1 Second sleep before next price query

struct MainState {
    market: Market,
    config: Config,
}

impl MainState {
    fn new(config: Config) -> MainState {
        MainState {
            market: Binance::new(None, None),
            config,
        }
    }

    fn main_loop(&mut self) {
        loop {
            println!("{:?}", self.market.get_price("ETHUSDT").expect("Could not get prices."));
            thread::sleep(PRICE_QUERY_SLEEP_TIME);
        }
    }

    #[inline]
    fn update_config<P: AsRef<Path> + Display>(&mut self, path: P) {
        self.config = Config::from_path(path);
    }
}

fn main() {
    let config = Config::from_path(CONFIG_PATH_STR);
    // println!("{:?}", config);
    let mut state = MainState::new(config);
    state.main_loop();
}