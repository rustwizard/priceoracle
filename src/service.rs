use clap::ArgMatches;
use hyper::{Body, Client, Request};
use hyper_tls::HttpsConnector;

pub fn run(logger: slog::Logger, arg: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new(arg);
    let url = make_url(config.api_endpoint.unwrap(), config.api_key.unwrap()).unwrap();
    info!(logger, "service called to the {}", url);

    return send_request(url);
}

struct Config {
    api_endpoint: Option<String>,
    api_key: Option<String>,
}

impl Config {
    fn new(arg: &ArgMatches) -> Self {
        let api_endpoint = arg.value_of("api_endpoint").unwrap().to_string();
        let api_key = arg.value_of("api_key").unwrap().to_string();

        Config {
            api_endpoint: Some(api_endpoint),
            api_key: Some(api_key),
        }
    }
}

fn make_url(api_endpoint: String, api_key: String) -> Option<String> {
    let mut url = api_endpoint + "/data/price?fsym=BTC&tsyms=USD,ETH";
    url = url + "&api_key=" + api_key.as_ref();
    Some(url)
}

#[tokio::main]
async fn send_request(url: String) -> Result<(), Box<dyn std::error::Error>> {
    let req = Request::builder().uri(url).body(Body::empty()).unwrap();
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let resp = client.request(req).await?;
    println!("Response: {}", resp.status());
    Ok(())
}
