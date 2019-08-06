use clap::ArgMatches;
use web3::contract::{Contract, Options};
use web3::types::{Address, U256, H256};
use web3::futures::Future;
use web3::transports::Http;
use std::vec::Vec;

use ethereum_types::{U256 as EU256};

#[derive(RustEmbed)]
#[folder = "src/contract/"]
struct Asset;

pub fn run(logger: slog::Logger, arg: &ArgMatches) -> Result<(), String> {
    let net = arg.value_of("net").unwrap();
    let newprice = arg.value_of("newprice").unwrap();
    let ca = arg.value_of("contractaddr").unwrap();

    let gas_limit = arg.value_of("gas_limit").unwrap();
    let ugas_limit: EU256 = EU256::from_dec_str(gas_limit).unwrap();

    let from_addr =  arg.value_of("from_addr").unwrap();
    let private_key = arg.value_of("private_key").unwrap();

    info!(logger, "updateprice called to the {} network with {} price and contractaddr {} and \
                    gas_limit {}",
          net, newprice, ca, ugas_limit);

    let contract_abi = Asset::get("PriceOracle.abi").unwrap();

    let contract_address: Address = ca.parse().unwrap();

    let (eloop, http) = web3::transports::Http::new(net).unwrap();
    eloop.into_remote();

    let web3 = web3::Web3::new(http);

    let price: U256 = newprice.parse().unwrap();

    let tx =  if from_addr.len() != 0 {
        match with_own_eth_node(web3, contract_address,
                                contract_abi.as_ref(), ugas_limit, price) {
            Err(e) => return Err(e.to_string()),
            Ok(tx) => tx,
        };
    } else {
        match with_existing_wallet() {
            Err(e) => return Err(e.to_string()),
            Ok(tx) => tx,
        };
    };


    info!(logger, "tx: {:?}", tx);

    Ok(())
}

fn with_existing_wallet() -> Result<(H256), String> {
    Ok(H256::zero())
}

fn with_own_eth_node(eth_client: web3::Web3<Http>,
                     contract_address: Address,
                     contract_abi: &[u8],
                     gas_limit: EU256,
                     newprice: U256) -> Result<(H256), String> {
    let accounts = eth_client.eth().accounts().wait().unwrap();

    if accounts.len() == 0 {
        return Err(String::from("there is no any accounts for contract deploy"))
    }

    let contract = Contract::from_json(
        eth_client.eth(),
        contract_address,
        contract_abi,
    ).unwrap();

    let result
        = contract.call("updatePrice", (newprice,), accounts[0].into(), Options::default());

    let tx = result.wait().unwrap();

    Ok(tx)
}