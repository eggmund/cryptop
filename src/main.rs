#[macro_use] extern crate log;

mod config;
mod market;

use tui::Terminal;
use tui::backend::{TermionBackend, Backend};
use tui::widgets::{Block, Borders, Chart, Axis, Dataset, GraphType};
use tui::layout::{Layout, Constraint, Direction};
use tui::symbols;
use tui::style::{Style, Color};

// use chrono::prelude::*;

use std::thread;
use std::time::Duration;
use std::fs::File;
use std::io;
use std::sync::Arc;
use std::time::Instant;

use config::Config;
use market::MarketData;

// backend used for tui
type TuiBackend = TermionBackend<io::Stdout>;

const CONFIG_PATH_STR: &'static str = "./config.yaml";
const LOG_PATH_STR: &'static str = "cryptop.log";
const UPDATE_SLEEP_TIME: Duration = Duration::from_millis(1000);

struct App {
    config: Arc<Config>,
    market_data: MarketData,
}

impl App {
    fn new(config: Config) -> App {
        Self::init_logger();
        let config = Arc::new(config);

        App {
            config: config.clone(),
            market_data: MarketData::new(config.clone(), ),
        }
    }

    fn main_loop(&mut self) {
        use ndarray::Array;

        let mut terminal = Self::init_terminal().unwrap();
        // clear screen
        terminal.backend_mut().clear().unwrap();

        loop {
            // UPDATE
            let update_start_time = Instant::now();
            self.market_data.update();

            let display_price = format!("{:.2}", self.market_data.price);
            // info!("Display price: {}", display_price);

            let price_dataset = [
                Dataset::default()
                    .name(&display_price)
                    .marker(symbols::Marker::Dot)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(Color::Cyan))
                    .data(&self.market_data.price_and_time_plot_data)
            ];

            let (price_min, price_max) = MarketData::min_max_price(&self.market_data.klines);
            let time_bounds = [self.market_data.price_and_time_plot_data[0].0, self.market_data.price_and_time_plot_data.last().unwrap().0];
            let price_bounds = [price_min, price_max];

            let price_labels: Vec<String> = Array::linspace(price_bounds[0], price_bounds[1], 10)
                .into_raw_vec().into_iter()
                .map(|n| format!("{:.2}", n))
                .collect();


            // DRAW
            terminal.draw(|mut f| {
                let size = f.size();  
                let main_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(100),
                        // Constraint::Percentage(20),
                    ].as_ref())
                    .split(size);

                let price_chart: Chart<'_, String, String> = Chart::default()
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .title(&display_price)
                    )
                    .x_axis(Axis::default()
                        .title("Time")
                        .bounds(time_bounds)
                    )
                    .y_axis(Axis::default()
                        .title("Price")
                        .bounds(price_bounds)
                        .labels(&price_labels)
                    )
                    .datasets(&price_dataset);
                
                f.render_widget(price_chart, main_layout[0]);
            }).unwrap();

            let frame_time = Instant::now().duration_since(update_start_time);
            info!("Frame time: {:?}", frame_time);
            if frame_time < UPDATE_SLEEP_TIME {
                thread::yield_now();
                thread::sleep(UPDATE_SLEEP_TIME - frame_time);
            }
        }
    }

    // #[inline]
    // fn update_config<P: AsRef<Path>>(&mut self, path: P) {
    //     self.config = Arc::new(Config::from_path(path));
    // }

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
    let mut state = App::new(config);
    state.main_loop();
}