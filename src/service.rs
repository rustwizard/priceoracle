#![allow(non_snake_case)]

use serde::Deserialize;

use clap::ArgMatches;
use hyper::Client;
use hyper_tls::HttpsConnector;
use std::{thread, time};

use bytes::buf::BufExt as _;

#[tokio::main]
pub async fn run(logger: slog::Logger, arg: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new(arg);
    let url = make_url(config.api_endpoint.unwrap(), config.api_key.unwrap()).unwrap();
    info!(
        logger,
        "service called to the {} with poll interval {}",
        url,
        config.poll_interval.unwrap()
    );
    loop {
        let price = fetch_json(url.as_str().parse().unwrap()).await?;
        info!(logger, "one BTC for ETH now is {:#?}", price);
        thread::sleep(time::Duration::from_secs(config.poll_interval.unwrap()));
    }
}

#[derive(Default)]
struct Config {
    api_endpoint: Option<String>,
    api_key: Option<String>,
    poll_interval: Option<u64>,
}

impl Config {
    fn new(arg: &ArgMatches) -> Self {
        let api_endpoint = arg.value_of("api_endpoint").unwrap().to_string();
        let api_key = arg.value_of("api_key").unwrap().to_string();
        let cpi = arg.value_of("poll_interval").unwrap();

        let poll_interval = cpi.parse::<u64>().unwrap();

        Config {
            api_endpoint: Some(api_endpoint),
            api_key: Some(api_key),
            poll_interval: Some(poll_interval),
        }
    }
}

fn make_url(api_endpoint: String, api_key: String) -> Option<String> {
    let mut url = api_endpoint + "/data/price?fsym=BTC&tsyms=ETH";
    url = url + "&api_key=" + api_key.as_ref();
    Some(url)
}

async fn fetch_json(url: hyper::Uri) -> Result<OneBtcToEth, Box<dyn std::error::Error>> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    // Fetch the url...
    let resp = client.get(url).await?;
    println!("Response: {}", resp.status());

    // asynchronously aggregate the chunks of the body
    let body = hyper::body::aggregate(resp).await?;
    let price = serde_json::from_reader(body.reader())?;

    Ok(price)
}

#[derive(Deserialize, Debug)]
struct OneBtcToEth {
    ETH: f64,
}
