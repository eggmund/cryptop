use binance::model::{KlineSummary, KlineSummaries};
use binance::market::Market;
use binance::api::*;

use chrono::prelude::*;
use chrono::{Timelike, Datelike};

use std::sync::Arc;

use crate::config::Config;

const INITIAL_CANDLE_NUM: u16 = 32;

// Stored data about the markets (updated in separate thread)
pub struct MarketData {
    config: Arc<Config>,
    market: Market,
    pub price: f64,
    pub klines: Vec<KlineSummary>,

    current_candle_interval: (String, u32),   // ("binance api string", value). E.g ("15m", 15)
    received_new_kline: bool,
    pub price_and_time_plot_data: Vec<(f64, f64)>
}

impl MarketData {
    pub fn new(config: Arc<Config>) -> MarketData {
        let market: Market = Binance::new(None, None);
        let price = market.get_price(&config.symbol).unwrap().price;
        let klines: Vec<KlineSummary> = match market.get_klines(&config.symbol, &config.candle_interval, INITIAL_CANDLE_NUM, None, None).unwrap() {
            KlineSummaries::AllKlineSummaries(klines) => klines,
        };
        let price_and_time_plot_data = Self::get_price_and_time_plot_data(&klines);

        let current_candle_interval = Self::get_candle_interval_from_string(&config.candle_interval);

        MarketData {
            config: config.clone(),
            market: market,
            price,
            klines,

            current_candle_interval,
            received_new_kline: true,
            price_and_time_plot_data,
        }
    }

    pub fn update(&mut self) {
        let datetime = Local::now();

        if self.get_time_element_to_watch(datetime) % self.current_candle_interval.1 == 0 { // If there is a new candle to receive
            if !self.received_new_kline {                // If latest not received yet
                match self.market.get_klines(&self.config.symbol, &self.current_candle_interval.0, 1, None, None).unwrap() {
                    KlineSummaries::AllKlineSummaries(new_klines) => {
                        let last = self.klines.last().unwrap();
                        if new_klines[0].open_time != last.open_time {
                            info!("Received new kline: {}", new_klines[0].open_time);
                            self.received_new_kline = true;      // got it
                            self.price_and_time_plot_data.push((last.close_time as f64, last.close));
                            self.klines.push(new_klines[0].clone());         // otherwise it will check on the next loop (clock may not be synced)
                        }
                    }
                }
            }
        } else if self.received_new_kline {    // No new one to receive so haven't received a new one yet
            self.received_new_kline = false;
        }

        self.price = self.market.get_price(&self.config.symbol).unwrap().price;
    }

    #[inline]
    fn get_time_element_to_watch(&self, datetime: DateTime<Local>) -> u32 {
        match self.current_candle_interval.0.chars().last().unwrap() {
            'm' => datetime.minute(),
            'h' => datetime.hour(),
            'd' => datetime.day(),
            x => panic!("Invalid candle time character: {}", x),
        }
    }

    #[inline]
    fn get_candle_interval_from_string(st: &str) -> (String, u32) {
        (st.into(), st.replace(['m', 'h', 'D', 'W', 'M'].as_ref(), "").parse::<u32>().unwrap())
    }

    pub fn get_price_and_time_plot_data(klines: &Vec<KlineSummary>) -> Vec<(f64, f64)> {
        let mut out: Vec<(f64, f64)> = Vec::with_capacity(klines.len());
        for k in klines.iter() {
            out.push((k.close_time as f64, k.close));
        }
        out
    }

    pub fn min_max_price(klines: &Vec<KlineSummary>) -> (f64, f64) {
        let mut min = std::f64::MAX;
        let mut max = 0.0;
        for k in klines.iter() {
            if k.close < min {
                min = k.close;
            } else if k.close > max {
                max = k.close;
            }
        }
        (min, max)
    }
}