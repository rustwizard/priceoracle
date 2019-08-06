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
    let _private_key = arg.value_of("private_key").unwrap();

    info!(logger, "updateprice called to the {} network with {} price and contractaddr {} and \
                    gas_limit {}",
          net, newprice, ca, ugas_limit);

    let contract_abi = Asset::get("PriceOracle.abi").unwrap();

    let contract_address: Address = ca.parse().unwrap();

    let (eloop, http) = web3::transports::Http::new(net).unwrap();
    eloop.into_remote();

    let web3 = web3::Web3::new(http);

    let price: U256 = newprice.parse().unwrap();


    let contract = Contract::from_json(
        web3.eth(),
        contract_address,
        contract_abi.as_ref(),
    ).unwrap();

    let tx =  if from_addr.len() == 0 {
        match with_own_eth_node(web3, contract,
                               ugas_limit, price) {
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

fn with_existing_wallet(
                        ) -> Result<(H256), String> {
    Ok(H256::zero())
}

fn with_own_eth_node(eth_client: web3::Web3<Http>,
                     contract: Contract<Http>,
                     gas_limit: EU256,
                     newprice: U256) -> Result<(H256), String> {
    let accounts = eth_client.eth().accounts().wait().unwrap();

    if accounts.len() == 0 {
        return Err(String::from("there is no any accounts for contract deploy"))
    }

    let options = if gas_limit.ne(&U256::zero()) {

        let gas_price = match eth_client.eth().gas_price().wait() {
            Ok(gas_price) => gas_price,
            Err(e) => return Err(e.to_string()),
        };

        Options {
            gas: Some(gas_limit),
            gas_price: Some(gas_price),
            value: None,
            nonce: None,
            condition: None
        }
    } else {
        Options::default()
    };

    let result
        = contract.call("updatePrice", (newprice,), accounts[0].into(), options.into());

    let tx = result.wait().unwrap();

    Ok(tx)
}