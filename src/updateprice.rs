use clap::ArgMatches;
use web3::contract::{Contract, Options};
use web3::types::{Address, U256};
use web3::futures::Future;
use std::vec::Vec;

#[derive(RustEmbed)]
#[folder = "src/contract/"]
struct Asset;

pub fn run(logger: slog::Logger, arg: &ArgMatches) -> Result<(), String> {
    let net = arg.value_of("net").unwrap();
    let newprice = arg.value_of("newprice").unwrap();
    let ca = arg.value_of("contractaddr").unwrap();
    info!(logger, "updateprice called to the {} network with {} price. contractaddr {}",
          net, newprice, ca);

    let contract_abi = Asset::get("PriceOracle.abi").unwrap();

    let contract_address: Address = ca.parse().unwrap();

    let (eloop, http) = web3::transports::Http::new(net).unwrap();
    eloop.into_remote();

    let web3 = web3::Web3::new(http);

    let accounts = web3.eth().accounts().wait().unwrap();

    if accounts.len() == 0 {
        return Err(String::from("there is no any accounts for contract deploy"))
    }

    let contract = Contract::from_json(
        web3.eth(),
        contract_address,
        contract_abi.as_ref(),
    ).unwrap();

    let price: U256 = newprice.parse().unwrap();
    let from_addr = accounts[0].into();

    let result
        = contract.call("updatePrice", (price,), from_addr, Options::default());

    let tx = result.wait().unwrap();

    info!(logger, "tx: {:?}", tx);

    Ok(())
}