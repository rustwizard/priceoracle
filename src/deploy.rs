use clap::ArgMatches;
use web3::contract::{Contract, Options};
use web3::futures::Future;
use web3::types::{U256, Address};
use web3::transports::Http;
use std::time::Duration;

use ethereum_types::H256;

#[derive(RustEmbed)]
#[folder = "src/contract/"]
struct Asset;

pub fn run(logger: slog::Logger, arg: &ArgMatches) -> Result<(), String> {
    let net = arg.value_of("net").unwrap();

    let from_addr =  arg.value_of("from_addr").unwrap();
    let private_key = arg.value_of("private_key").unwrap();

    info!(logger, "deploy called to the {} network with {}", net, from_addr);

    let (eloop, http) = web3::transports::Http::new(net).unwrap();
    eloop.into_remote();

    let web3 = web3::Web3::new(http);

    let contract_address =  if from_addr.len() != 0 {
        match with_existing_wallet(web3, &logger, from_addr, private_key) {
            Err(e) => return Err(e.to_string()),
            Ok(a) => a,
        };
    } else {
        match with_own_eth_node(web3, &logger) {
            Err(e) => return Err(e.to_string()),
            Ok(a) => a,
        };
    };

    info!(logger,"contract address: {:?}", contract_address);

    Ok(())
}

fn with_existing_wallet(eth_client: web3::Web3<Http>,
                        logger: &slog::Logger,
                        from_addr: &str,
                        private_key: &str) -> Result<(Address), String> {
    let contract_abi = Asset::get("PriceOracle.abi").unwrap();
    info!(logger, "{:?}", std::str::from_utf8(contract_abi.as_ref()));

    let contract_bytecode = Asset::get("PriceOracle.bin").unwrap();

    let gas_price: U256 = eth_client.eth().gas_price().wait().unwrap();

    info!(logger,"deploy contract from {} with suggested gas_price: {:?}", from_addr, gas_price);
    let my_account: Address = from_addr.parse().unwrap();
    let nonce  =
        eth_client.eth().transaction_count(my_account, None);
    let tx_request = ethtxsign::RawTransaction {
        to: None,
        gas: 210_000.into(),
        gas_price: 1_000_000_000.into(),
        value: 0.into(),
        data: contract_bytecode.into(),
        nonce: nonce.wait().unwrap(),
    };

    let pk = pvt_key_from_slice(hex::decode(private_key.as_bytes()).unwrap().as_slice()).unwrap();
    let tx = tx_request.sign(&pk.into(), &3.into());

    let result =
        eth_client.send_raw_transaction_with_confirmation(tx.into(),Duration::from_secs(1), 1);
    let receipt = result.wait().unwrap();
    info!(logger, "tx {} created", receipt.transaction_hash);

    Ok("contract_address".parse().unwrap())
}

fn with_own_eth_node(eth_client: web3::Web3<Http>, logger: &slog::Logger) -> Result<(Address), String> {
    let accounts = eth_client.eth().accounts().wait().unwrap();

    if accounts.len() == 0 {
        return Err(String::from("there is no any accounts for contract deploy"))
    }


    let contract_abi = Asset::get("PriceOracle.abi").unwrap();
    info!(logger, "{:?}", std::str::from_utf8(contract_abi.as_ref()));

    let contract_bytecode = Asset::get("PriceOracle.bin").unwrap();

    info!(logger,"Accounts: {:?}", accounts);
    let gas_price: U256 = eth_client.eth().gas_price().wait().unwrap();

    info!(logger,"suggested gas_price: {:?}", gas_price);

    let bc = std::str::from_utf8(contract_bytecode.as_ref()).unwrap();

    let contract = Contract::deploy(eth_client.eth(), contract_abi.as_ref())
        .unwrap()
        .confirmations(0)
        .options(Options::with(|opt| {
            opt.value = Some(0.into());
            opt.gas_price = Some(gas_price);
            opt.gas = Some(1_000_000.into());
        }))
        .execute(
            bc,
            (),
            accounts[0],
        )
        .expect("Correct parameters are passed to the constructor.")
        .wait()
        .unwrap();

    let contract_address = contract.address();

    Ok(contract_address)
}

fn pvt_key_from_slice(key: &[u8]) -> Option<H256> {
    if key.len() != 32 {
        return None
    }
    let mut h = H256::zero();
    h.as_bytes_mut().copy_from_slice(&key[0..32]);
    Some(h)
}

