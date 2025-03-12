use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const ALPHA_VANTAGE_API_KEY: &str = "F3ZZV8LPPY32GSA0";
static STOCK_SERVICE: OnceLock<StockService> = OnceLock::new();
const CACHE_FILE_PATH: &str = "stock.cache.json";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StockHistoricalData {
    pub dates: Vec<String>,
    pub prices: Vec<f64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StockQuote {
    pub symbol: String,
    pub current_price: f64,
    pub change: f64,
    pub change_percent: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SerializableStockData {
    pub quote: StockQuote,
    pub historical_data: StockHistoricalData,
    pub fetched_at_timestamp: u64,
}

#[derive(Clone, Debug)]
pub struct StockData {
    pub quote: StockQuote,
    pub historical_data: StockHistoricalData,
    pub fetched_at: Instant,
}
pub struct StockService {
    client: reqwest::blocking::Client,
    cached_data: std::sync::Mutex<Option<(StockData, StockData)>>,
}

impl StockData {
    fn to_serializable(&self) -> SerializableStockData {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let elapsed = self.fetched_at.elapsed().as_secs();
        let timestamp = now - elapsed;

        SerializableStockData {
            quote: self.quote.clone(),
            historical_data: self.historical_data.clone(),
            fetched_at_timestamp: timestamp,
        }
    }
}

impl SerializableStockData {
    fn to_stock_data(&self) -> StockData {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let age = now - self.fetched_at_timestamp;
        let fetched_at = Instant::now() - Duration::from_secs(age);

        StockData {
            quote: self.quote.clone(),
            historical_data: self.historical_data.clone(),
            fetched_at,
        }
    }
}

impl StockService {
    pub fn instance() -> &'static StockService {
        STOCK_SERVICE.get_or_init(|| StockService::new())
    }

    pub fn new() -> Self {
        let cached_data = Self::load_cache_from_disk()
            .map(|data| std::sync::Mutex::new(Some(data)))
            .unwrap_or_else(|_| std::sync::Mutex::new(None));

        Self {
            client: reqwest::blocking::Client::new(),
            cached_data,
        }
    }

    fn load_cache_from_disk() -> Result<(StockData, StockData), Box<dyn std::error::Error>> {
        if !Path::new(CACHE_FILE_PATH).exists() {
            return Err("Cache file not found".into());
        }

        let file_content = fs::read_to_string(CACHE_FILE_PATH)?;
        let serialized_data: (SerializableStockData, SerializableStockData) =
            serde_json::from_str(&file_content)?;

        let wba_data = serialized_data.0.to_stock_data();
        let cvs_data = serialized_data.1.to_stock_data();
        Ok((wba_data, cvs_data))
    }

    fn save_cache_to_disk(
        &self,
        wba_data: &StockData,
        cvs_data: &StockData,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let serializable_wba = wba_data.to_serializable();
        let serializable_cvs = cvs_data.to_serializable();

        let serialized_data = serde_json::to_string(&(serializable_wba, serializable_cvs))?;
        fs::write(CACHE_FILE_PATH, serialized_data)?;

        Ok(())
    }

    pub fn fetch_stock_data(&self) -> Result<(StockData, StockData), Box<dyn std::error::Error>> {
        println!("Fetching stock Data");

        let wba_quote = self.fetch_single_stock_quote("WBA")?;
        let wba_historical = self.fetch_single_historical_data("WBA")?;
        let wba_data = StockData {
            quote: wba_quote,
            historical_data: wba_historical,
            fetched_at: Instant::now(),
        };

        let cvs_quote = self.fetch_single_stock_quote("CVS")?;
        let cvs_historical = self.fetch_single_historical_data("CVS")?;
        let cvs_data = StockData {
            quote: cvs_quote,
            historical_data: cvs_historical,
            fetched_at: Instant::now(),
        };

        if let Err(e) = self.save_cache_to_disk(&wba_data, &cvs_data) {
            println!("Failed to save cache: {}", e);
        } else {
            println!("Saved cache to disk");
        }
        Ok((wba_data, cvs_data))
    }

    pub fn get_stock_data(&self) -> Result<(StockData, StockData), Box<dyn std::error::Error>> {
        {
            let cache = self.cached_data.lock().unwrap();
            if let Some((wba_data, cvs_data)) = &*cache {
                //println!("wba_data.fetched_at {:?}", wba_data.fetched_at);
                if wba_data.fetched_at.elapsed() < Duration::from_secs(24 * 60 * 60) {
                    //println!("Cached stock data is still fresh");
                    return Ok((wba_data.clone(), cvs_data.clone()));
                }
            }
        }

        println!("Cached data is stale, fetching new");
        let stock_data = self.fetch_stock_data()?;
        {
            let mut cache = self.cached_data.lock().unwrap();
            *cache = Some(stock_data.clone());
        }

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

        //we should only get here one time
        println!("fetching: {}", url);
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
