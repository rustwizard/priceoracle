use clap::ArgMatches;
use web3::contract::{Contract, Options};
use web3::types::{Address, U256, H256};
use web3::futures::Future;
use web3::transports::Http;
use std::vec::Vec;
use std::time::Duration;

#[derive(RustEmbed)]
#[folder = "src/contract/"]
struct Asset;

pub fn run(logger: slog::Logger, arg: &ArgMatches) -> Result<(), String> {
    let config = UpdateConfig::new(arg);

    info!(logger, "updateprice called to the {} network with {} price and contractaddr {} and \
                    gas_limit {}",
          config.net, config.new_price, config.contract_addr.unwrap(), config.gas_limit);

    let (eloop, http) = web3::transports::Http::new(&config.net).unwrap();
    eloop.into_remote();

    let web3 = web3::Web3::new(http);

    let tx = match config.from_addr {
        None => with_own_eth_node(web3, config),
        Some(_) => with_existing_wallet(web3, config),
    };


    info!(logger, "tx: {:?}", tx);

    Ok(())
}

fn with_existing_wallet(eth_client: web3::Web3<Http>, conf: UpdateConfig) -> Result<(H256), String> {
    let method_id = ethtxsign::keccak256_hash(b"updatePrice(uint256)");
    let update_price_abi = format!("{}{:064x}", &hex::encode(method_id)[..8],
                                   &conf.new_price.as_u64());
    println!("update_price_abi {}", update_price_abi);

    let nonce_cnt = match eth_client.eth().transaction_count(conf.from_addr.unwrap(), None).wait() {
        Ok(nonce) => nonce,
        Err(e) => return Err(e.to_string()),
    };

    let gas_price = match eth_client.eth().gas_price().wait() {
        Ok(gas_price) => gas_price,
        Err(e) => return Err(e.to_string()),
    };

    let data = hex::decode(update_price_abi.as_bytes());

    let tx_request = ethtxsign::RawTransaction {
        to: conf.contract_addr,
        gas: conf.gas_limit,
        gas_price: gas_price.into(),
        value: 0.into(),
        data: data.unwrap(),
        nonce: nonce_cnt,
    };

    let tx = tx_request.sign(&conf.pvt_key, &conf.chain_id);

    let result =
        eth_client.send_raw_transaction_with_confirmation(tx.into(), Duration::from_secs(1), 1);

    let receipt = match result.wait() {
        Ok(receipt) => receipt,
        Err(e) => return Err(e.to_string()),
    };

    Ok(receipt.transaction_hash)
}

fn with_own_eth_node(eth_client: web3::Web3<Http>, conf: UpdateConfig) -> Result<(H256), String> {
    let contract = Contract::from_json(
        eth_client.eth(),
        conf.contract_addr.unwrap(),
        conf.contract_abi.as_slice(),
    ).unwrap();

    let accounts = eth_client.eth().accounts().wait().unwrap();

    if accounts.len() == 0 {
        return Err(String::from("there is no any accounts for contract deploy"));
    }

    let options = if conf.gas_limit.ne(&U256::zero()) {
        let gas_price = match eth_client.eth().gas_price().wait() {
            Ok(gas_price) => gas_price,
            Err(e) => return Err(e.to_string()),
        };

        Options {
            gas: Some(conf.gas_limit),
            gas_price: Some(gas_price),
            value: None,
            nonce: None,
            condition: None,
        }
    } else {
        Options::default()
    };

    let result
        = contract.call("updatePrice", (conf.new_price, ), accounts[0].into(), options.into());

    let tx = result.wait().unwrap();

    Ok(tx)
}

struct UpdateConfig {
    from_addr: Option<Address>,
    contract_addr: Option<Address>,
    new_price: U256,
    pvt_key: H256,
    gas_limit: U256,
    contract_bytecode: Vec<u8>,
    contract_abi: Vec<u8>,
    chain_id: u8,
    net: String,
}

impl UpdateConfig {
    fn new(arg: &ArgMatches) -> Self {
        let net = arg.value_of("net").unwrap().to_string();

        let my_account = arg.value_of("from_addr").unwrap();
        let fr: Address = my_account.parse().unwrap();

        let ca = arg.value_of("contractaddr").unwrap();
        let contract_address: Address = ca.parse().unwrap();

        let np = arg.value_of("newprice").unwrap();
        let new_price = U256::from_dec_str(np).unwrap();

        let pk = arg.value_of("private_key").unwrap();
        let pvt_key = ethtxsign::pvt_key_from_slice(hex::decode(pk.as_bytes()).unwrap().as_slice()).unwrap();

        let gl = arg.value_of("gas_limit").unwrap();
        let gas_limit: U256 = U256::from_dec_str(gl).unwrap();

        let cid = arg.value_of("chain_id").unwrap();
        let chain_id = cid.parse::<u8>().unwrap();

        let cb = Asset::get("PriceOracle.bin").unwrap();
        let contract_bytecode = hex::decode(cb.as_ref()).unwrap();

        let cabi = Asset::get("PriceOracle.abi").unwrap();
        let contract_abi = cabi.as_ref().to_vec();

        UpdateConfig {
            from_addr: Some(fr),
            contract_addr: Some(contract_address),
            new_price,
            pvt_key,
            gas_limit,
            contract_bytecode,
            contract_abi,
            chain_id,
            net,
        }
    }
}