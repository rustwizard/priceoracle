use clap::ArgMatches;
use web3::types::{Address};

#[derive(RustEmbed)]
#[folder = "src/contract/"]
struct Asset;

pub fn run_with_ws(logger: slog::Logger, arg: &ArgMatches) -> Result<(), String> {
    let config = Config::new(arg);

    info!(
        logger,
        "readevent runs on the {} network with contractaddr {}",
        config.net,
        config.contract_addr.unwrap(),
    );

    let (eloop, http) = web3::transports::WebSocket::new(&config.net).unwrap();
    eloop.into_remote();

    let _ = web3::Web3::new(http);



    Ok(())
}

struct Config {
    contract_addr: Option<Address>,
    contract_abi: Vec<u8>,
    net: String,
    block_num: i64,
}

impl Config {
    fn new(arg: &ArgMatches) -> Self {
        let net = arg.value_of("net").unwrap().to_string();


        let ca = arg.value_of("contractaddr").unwrap();
        let contract_address: Address = ca.parse().unwrap();


        let cabi = Asset::get("PriceOracle.abi").unwrap();
        let contract_abi = cabi.as_ref().to_vec();

        let block_num: i64 = arg.value_of("blocknum").unwrap().parse().unwrap();

        Config {
            contract_addr: Some(contract_address),
            contract_abi,
            net,
            block_num,
        }
    }
}
