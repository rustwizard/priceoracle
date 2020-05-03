use clap::ArgMatches;

pub fn run(logger: slog::Logger, arg: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new(arg);
    info!(
        logger,
        "service called to the {} endpoint with the {:?} api key",
        config.api_endpoint.unwrap(),
        config.api_key.unwrap()
    );
    Ok(())
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
