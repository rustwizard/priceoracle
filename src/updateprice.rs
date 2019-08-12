use clap::ArgMatches;
use web3::contract::{Contract, Options};
use web3::types::{Address, U256, H256};
use web3::futures::Future;
use web3::transports::Http;
use std::vec::Vec;
use ethereum_types::{U256 as EU256};
use std::time::Duration;

#[derive(RustEmbed)]
#[folder = "src/contract/"]
struct Asset;

pub fn run(logger: slog::Logger, arg: &ArgMatches) -> Result<(), String> {
    let net = arg.value_of("net").unwrap();
    let newprice = arg.value_of("newprice").unwrap();
    let ca = arg.value_of("contractaddr").unwrap();

    let chain_id = arg.value_of("chain_id").unwrap();

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

    let price = U256::from_dec_str(newprice).unwrap();

    let tx =  if from_addr.len() == 0 {
        match with_own_eth_node(web3, contract_address, contract_abi.as_ref(),
                               ugas_limit, price) {
            Err(e) => return Err(e.to_string()),
            Ok(tx) => tx,
        };
    } else {
        match with_existing_wallet(web3, contract_address, from_addr,private_key,
                                   &chain_id.parse::<u8>().unwrap(), ugas_limit, price) {
            Err(e) => return Err(e.to_string()),
            Ok(tx) => tx,
        };
    };


    info!(logger, "tx: {:?}", tx);

    Ok(())
}

fn with_existing_wallet(eth_client: web3::Web3<Http>,
                        contract_address: Address,
                        from_addr: &str,
                        private_key: &str,
                        chain_id : &u8,
                        gas_limit: EU256,
                        newprice: EU256) -> Result<(H256), String> {

    let my_account: Address = match from_addr.parse() {
        Ok(from_addr) => from_addr,
        Err(e) => return Err(e.to_string()),
    };

    let method_id = ethtxsign::keccak256_hash(b"updatePrice(uint256)");
    let update_price_abi = format!("{}{:064x}", &hex::encode(method_id)[..8],
                                   U256::as_u64(&newprice));
    println!("update_price_abi {}", update_price_abi);

    let nonce_cnt  = match eth_client.eth().transaction_count(my_account, None).wait() {
        Ok(nonce) => nonce,
        Err(e) => return Err(e.to_string()),
    };

    let gas_price = match eth_client.eth().gas_price().wait() {
        Ok(gas_price) => gas_price,
        Err(e) => return Err(e.to_string()),
    };

    let data = hex::decode(update_price_abi.as_bytes());

    let tx_request = ethtxsign::RawTransaction {
        to: Some(contract_address),
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

    Ok(receipt.transaction_hash)
}

fn with_own_eth_node(eth_client: web3::Web3<Http>,
                     contract_address: Address,
                     contract_abi: &[u8],
                     gas_limit: EU256,
                     newprice: U256) -> Result<(H256), String> {

    let contract = Contract::from_json(
        eth_client.eth(),
        contract_address,
        contract_abi,
    ).unwrap();

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