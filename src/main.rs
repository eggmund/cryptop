#[macro_use] extern crate log; 

mod config;

use binance::api::*;
use binance::market::*;

use log::Level;

use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{Widget};
use tui::layout::{Layout};

use std::thread;
use std::time::Duration;
use std::path::Path;
use std::fmt::Display;
use std::fs::File;
use std::io;

use config::Config;

// backend used for tui
type TuiBackend = TermionBackend<io::Stdout>;

const CONFIG_PATH_STR: &'static str = "./config.yaml";
const LOG_PATH_STR: &'static str = "cryptop.log";
const PRICE_QUERY_SLEEP_TIME: Duration = Duration::from_secs(1);       // 1 Second sleep before next price query

struct MainState {
    market: Market,
    config: Config,
    terminal: Terminal<TuiBackend>,
}

impl MainState {
    fn new(config: Config) -> Result<MainState, io::Error> {
        Self::init_logger();
        let terminal = Self::init_terminal()?;

        Ok(MainState {
            market: Binance::new(None, None),
            config,
            terminal,
        })
    }

    fn main_loop(&mut self) {
        loop {
            info!("{:?}", self.market.get_price("ETHUSDT").expect("Could not get prices."));
            thread::sleep(PRICE_QUERY_SLEEP_TIME);
        }
    }

    #[inline]
    fn update_config<P: AsRef<Path> + Display>(&mut self, path: P) {
        self.config = Config::from_path(path);
    }

    fn init_logger() {
        use simplelog::*;

        CombinedLogger::init(
            vec![
                TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed).unwrap(),
                WriteLogger::new(LevelFilter::Info, Config::default(), File::create(LOG_PATH_STR).unwrap()),
            ]
        ).unwrap();
    }

    fn init_terminal() -> Result<Terminal<TuiBackend>, io::Error> {
        let stdout = io::stdout();
        let backend = TuiBackend::new(stdout);
        Terminal::new(backend)
    }
}

fn main() {
    let config = Config::from_path(CONFIG_PATH_STR);
    // println!("{:?}", config);
    let mut state = MainState::new(config).unwrap();
    state.main_loop();
}