use clap::ArgMatches;
use web3::types::{Address};

#[derive(RustEmbed)]
#[folder = "src/contract/"]
struct Asset;

struct Config {
    contract_addr: Option<Address>,
    contract_abi: Vec<u8>,
    net: String,
}

impl Config {
    fn new(arg: &ArgMatches) -> Self {
        let net = arg.value_of("net").unwrap().to_string();


        let ca = arg.value_of("contractaddr").unwrap();
        let contract_address: Address = ca.parse().unwrap();


        let cabi = Asset::get("PriceOracle.abi").unwrap();
        let contract_abi = cabi.as_ref().to_vec();

        Config {
            contract_addr: Some(contract_address),
            contract_abi,
            net,
        }
    }
}
