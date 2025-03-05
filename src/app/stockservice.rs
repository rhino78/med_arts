use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

const ALPHA_VANTAGE_API_KEY: &str = "F3ZZV8LPPY32GSA0";

#[derive(Clone, Debug)]
pub struct StockHistoricalData {
    pub dates: Vec<String>,
    pub prices: Vec<f64>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct StockQuote {
    pub symbol: String,
    pub current_price: f64,
    pub change: f64,
    pub change_percent: f64,
}

#[derive(Clone, Debug)]
pub struct StockData {
    pub quote: StockQuote,
    pub historical_data: StockHistoricalData,
    pub fetched_at: Instant,
}
pub struct StockService {
    client: reqwest::blocking::Client,
    cached_data: OnceLock<(StockData, StockData)>,
}

impl StockService {
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
            cached_data: OnceLock::new(),
        }
    }

    pub fn fetch_stock_data(&self) -> Result<(StockData, StockData), Box<dyn std::error::Error>> {
        //Walgreens data
        let wba_quote = self.fetch_single_stock_quote("WBA")?;
        let wba_historical = self.fetch_single_historical_data("WBA")?;
        let wba_data = StockData {
            quote: wba_quote,
            historical_data: wba_historical,
            fetched_at: Instant::now(),
        };
        //CVS data
        let cvs_quote = self.fetch_single_stock_quote("CVS")?;
        let cvs_historical = self.fetch_single_historical_data("CVS")?;
        let cvs_data = StockData {
            quote: cvs_quote,
            historical_data: cvs_historical,
            fetched_at: Instant::now(),
        };

        Ok((wba_data, cvs_data))
    }

    pub fn get_stock_data(&self) -> Result<(StockData, StockData), Box<dyn std::error::Error>> {
        if let Some((wba_data, cvs_data)) = self.cached_data.get() {
            if wba_data.fetched_at.elapsed() < Duration::from_secs(24 * 60 * 60) {
                return Ok((wba_data.clone(), cvs_data.clone()));
            }
        }

        let stock_data = self.fetch_stock_data()?;
        self.cached_data.get_or_init(|| stock_data.clone());
        Ok(stock_data)
    }

    fn fetch_single_historical_data(
        &self,
        symbol: &str,
    ) -> Result<StockHistoricalData, Box<dyn std::error::Error>> {
        let url = format!(
            "https://www.alphavantage.co/query?function=GLOBAL_QUOTE&symbol={}&apikey={}",
            symbol, ALPHA_VANTAGE_API_KEY
        );

        let response = self.client.get(&url).send()?;
        let json: Value = response.json()?;

        let time_series = &json["Time Series (Daily)"];
        let mut dates = Vec::new();
        let mut prices = Vec::new();

        if let Some(obj) = time_series.as_object() {
            for (date, data) in obj.iter().take(5) {
                dates.push(date.to_string());
                prices.push(
                    data["4. close"]
                        .as_str()
                        .unwrap_or("0")
                        .parse()
                        .unwrap_or(0.0),
                );
            }
        }
        Ok(StockHistoricalData { dates, prices })
    }

    fn fetch_single_stock_quote(
        &self,
        symbol: &str,
    ) -> Result<StockQuote, Box<dyn std::error::Error>> {
        let url = format!(
            "https://www.alphavantage.co/query?function=GLOBAL_QUOTE&symbol={}&apikey={}",
            symbol, ALPHA_VANTAGE_API_KEY
        );

        let response = self.client.get(&url).send()?;
        let json: Value = response.json()?;
        let foo = json.get("Information");
        let yes = json.to_string();

        if yes.contains("Information") {
            Ok(StockQuote {
                symbol: "Foo".to_string(),
                current_price: 0.0,
                change: 0.0,
                change_percent: 0.0,
            })
        } else {
            let quote_data = &json["Global Quote"];
            Ok(StockQuote {
                symbol: quote_data["01. symbol"]
                    .as_str()
                    .unwrap_or(symbol)
                    .to_string(),
                current_price: quote_data["05. price"].as_str().unwrap_or("0").parse()?,
                change: quote_data["09. change"].as_str().unwrap_or("0'").parse()?,
                change_percent: quote_data["10. change percent"]
                    .as_str()
                    .unwrap_or("0")
                    .trim_end_matches('%')
                    .parse()?,
            })
        }
    }

    pub fn get_stock_quote(&self, symbol: &str) -> Result<StockQuote, reqwest::Error> {
        Ok(StockQuote {
            symbol: symbol.to_string(),
            current_price: match symbol {
                "WBA" => 23.45,
                "CVS" => 65.32,
                _ => 0.0,
            },
            change: match symbol {
                "WBA" => 0.55,
                "CVS" => -1.23,
                _ => 0.0,
            },
            change_percent: match symbol {
                "WBA" => 2.35,
                "CVS" => -1.88,
                _ => 0.0,
            },
        })
    }
}
