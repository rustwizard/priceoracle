#![allow(non_snake_case)]

use serde::Deserialize;

use clap::ArgMatches;
use hyper::Client;
use hyper_tls::HttpsConnector;
use std::{thread, time};

use crate::updateprice;
use bytes::buf::BufExt as _;
use web3::types::{U128, U256};

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

    let mut update_conf = updateprice::UpdateConfig::new(arg);
    let mut prev_price = 0.0;

    loop {
        let price = fetch_eth_price(url.as_str().parse().unwrap()).await?;
        info!(logger, "one BTC for ETH now is {:#?}", price);

        if price.ETH > prev_price {
            let wei = price.ETH * f64::powi(10.0, 18).ceil();
            let wei = wei as u128;
            update_conf.new_price = <U256 as From<U128>>::from(wei.into());
            if let Err(e) = updateprice::update_price(&logger, &update_conf) {
                info!(logger, "update price error: {:#?}", e);
            }
            prev_price = price.ETH;
        }

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

async fn fetch_eth_price(url: hyper::Uri) -> Result<OneBtcToEth, Box<dyn std::error::Error>> {
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
