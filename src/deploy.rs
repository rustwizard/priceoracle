use clap::ArgMatches;
use web3::contract::{Contract, Options};
use web3::futures::Future;
use web3::types::{U256, Address};
use web3::transports::Http;
use std::time::Duration;

use ethereum_types::{U256 as EU256};

#[derive(RustEmbed)]
#[folder = "src/contract/"]
struct Asset;

pub fn run(logger: slog::Logger, arg: &ArgMatches) -> Result<(), String> {
    let net = arg.value_of("net").unwrap();

    let from_addr =  arg.value_of("from_addr").unwrap();
    let private_key = arg.value_of("private_key").unwrap();

    let gas_limit = arg.value_of("gas_limit").unwrap();
    let ugas_limit: EU256 = EU256::from_dec_str(gas_limit).unwrap();
    let chain_id = arg.value_of("chain_id").unwrap();
    info!(logger, "deploy called to the {} network with {}", net, from_addr);

    let (eloop, http) = web3::transports::Http::new(net).unwrap();
    eloop.into_remote();

    let web3 = web3::Web3::new(http);

    let contract_address =  if from_addr.len() != 0 {
        match with_existing_wallet(web3,
                                   &logger,
                                   from_addr,
                                   private_key,
                                   &chain_id.parse::<u8>().unwrap(),
                                   ugas_limit) {
            Err(e) => return Err(e.to_string()),
            Ok(a) => a,
        };
    } else {
        match with_own_eth_node(web3, &logger, ugas_limit) {
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
                        private_key: &str,
                        chain_id : &u8,
                        gas_limit: EU256) -> Result<(Address), String> {
    let contract_abi = Asset::get("PriceOracle.abi").unwrap();
    info!(logger, "{:?}", std::str::from_utf8(contract_abi.as_ref()));

    let contract_bytecode = Asset::get("PriceOracle.bin").unwrap();

    let gas_price = match eth_client.eth().gas_price().wait() {
        Ok(gas_price) => gas_price,
        Err(e) => return Err(e.to_string()),
    };

    info!(logger,"deploy contract from {} with suggested gas_price: {:?}",
          from_addr, gas_price);

    let data = hex::decode(contract_bytecode.as_ref());

    let my_account: Address = match from_addr.parse() {
        Ok(from_addr) => from_addr,
        Err(e) => return Err(e.to_string()),
    };

    let nonce_cnt  = match eth_client.eth().transaction_count(my_account, None).wait() {
        Ok(nonce) => nonce,
        Err(e) => return Err(e.to_string()),
    };

    let tx_request = ethtxsign::RawTransaction {
        to: None,
        gas: gas_limit.into(),
        gas_price: gas_price.into(),
        value: 0.into(),
        data: data.unwrap(),
        nonce: nonce_cnt,
    };

    let pk = ethtxsign::pvt_key_from_slice(hex::decode(private_key.as_bytes()).unwrap().as_slice()).unwrap();
    let tx = tx_request.sign(&pk.into(), chain_id);

    let result =
        eth_client.send_raw_transaction_with_confirmation(tx.into(),Duration::from_secs(1), 1);

    let receipt = match result.wait() {
        Ok(receipt) => receipt,
        Err(e) => return Err(e.to_string()),
    };

    info!(logger, "tx {} created", receipt.transaction_hash);

    //TODO: return actual contract address
    Ok("contract_address".parse().unwrap())
}

fn with_own_eth_node(eth_client: web3::Web3<Http>, logger: &slog::Logger, gas_limit: EU256) -> Result<(Address), String> {
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
            opt.gas = Some(gas_limit.into());
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



