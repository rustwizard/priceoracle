use crate::web3util;
use clap::ArgMatches;
use core::fmt;
use std::convert::TryFrom;
use std::time::Duration;
use std::vec::Vec;
use web3::contract::{Contract, Options};
use web3::futures::Future;
use web3::types::{Address, H256, U256};
use web3::Transport;

#[derive(RustEmbed)]
#[folder = "src/contract/"]
struct Asset;

pub fn run_with_ws(
    logger: slog::Logger,
    arg: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = UpdateConfig::new(arg);

    info!(
        logger,
        "updateprice called to the {} network with {} price and contractaddr {} and \
         gas_limit {}",
        config.net,
        config.new_price,
        config.contract_addr.unwrap(),
        config.gas_limit
    );

    let (eloop, http) = web3::transports::WebSocket::new(&config.net).unwrap();
    eloop.into_remote();

    let web3 = web3::Web3::new(http);

    let tx = match config.from_addr {
        None => with_own_eth_node(web3, config),
        Some(_) => with_existing_wallet(web3, config),
    };

    info!(logger, "tx: {:?}", tx);

    Ok(())
}

pub fn run_with_http(
    logger: slog::Logger,
    arg: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = UpdateConfig::new(arg);

    info!(
        logger,
        "updateprice called to the {} network with {} price and contractaddr {} and \
         gas_limit {}",
        config.net,
        config.new_price,
        config.contract_addr.unwrap(),
        config.gas_limit
    );

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

fn with_existing_wallet(
    eth_client: web3::Web3<impl Transport>,
    conf: UpdateConfig,
) -> Result<H256, Box<dyn std::error::Error>> {
    let method_id = ethtxsign::keccak256_hash(b"updatePrice(uint256)");
    let update_price_abi = format!(
        "{}{:064x}",
        &hex::encode(method_id)[..8],
        &conf.new_price.as_u64()
    );
    println!("update_price_abi {}", update_price_abi);

    let nonce_cnt = web3util::nonce(conf.from_addr.unwrap(), &eth_client).unwrap();

    let gas_price = match eth_client.eth().gas_price().wait() {
        Ok(gas_price) => gas_price,
        Err(e) => return Err(Box::try_from(e).unwrap()),
    };

    let cdata = hex::decode(update_price_abi.as_bytes());

    let tx_request = ethtxsign::RawTransaction {
        to: conf.contract_addr,
        gas: conf.gas_limit,
        gas_price: gas_price.into(),
        value: 0.into(),
        data: cdata.unwrap(),
        nonce: nonce_cnt,
    };

    let tx = tx_request.sign(&conf.pvt_key, &conf.chain_id);

    let result =
        eth_client.send_raw_transaction_with_confirmation(tx.into(), Duration::from_secs(1), 1);

    let receipt = match result.wait() {
        Ok(receipt) => receipt,
        Err(e) => return Err(Box::try_from(e).unwrap()),
    };

    Ok(receipt.transaction_hash)
}

fn with_own_eth_node(
    eth_client: web3::Web3<impl Transport>,
    conf: UpdateConfig,
) -> Result<H256, Box<dyn std::error::Error>> {
    let contract = Contract::from_json(
        eth_client.eth(),
        conf.contract_addr.unwrap(),
        conf.contract_abi.as_slice(),
    )
    .unwrap();

    let accounts = eth_client.eth().accounts().wait().unwrap();

    if accounts.len() == 0 {
        return Err(
            Box::try_from(String::from("there is no any accounts for contract deploy")).unwrap(),
        );
    }

    let options = if conf.gas_limit.ne(&U256::zero()) {
        let gas_price = match eth_client.eth().gas_price().wait() {
            Ok(gas_price) => gas_price,
            Err(e) => return Err(Box::try_from(e).unwrap()),
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

    let result = contract.call(
        "updatePrice",
        (conf.new_price,),
        accounts[0].into(),
        options.into(),
    );

    let tx = result.wait().unwrap();

    Ok(tx)
}

pub fn update(logger: slog::Logger, conf: UpdateConfig) -> Result<(), Box<dyn std::error::Error>> {
    info!(logger, "config: {}", conf);
    Ok(())
}

pub struct UpdateConfig {
    from_addr: Option<Address>,
    contract_addr: Option<Address>,
    new_price: U256,
    pvt_key: H256,
    gas_limit: U256,
    contract_abi: Vec<u8>,
    chain_id: u8,
    net: String,
}

impl fmt::Display for UpdateConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({}, {}, {})",
            self.from_addr.unwrap(),
            self.contract_addr.unwrap(),
            self.new_price
        )
    }
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
        let pvt_key =
            ethtxsign::pvt_key_from_slice(hex::decode(pk.as_bytes()).unwrap().as_slice()).unwrap();

        let gl = arg.value_of("gas_limit").unwrap();
        let gas_limit: U256 = U256::from_dec_str(gl).unwrap();

        let cid = arg.value_of("chain_id").unwrap();
        let chain_id = cid.parse::<u8>().unwrap();

        let cabi = Asset::get("PriceOracle.abi").unwrap();
        let contract_abi = cabi.as_ref().to_vec();

        UpdateConfig {
            from_addr: Some(fr),
            contract_addr: Some(contract_address),
            new_price,
            pvt_key,
            gas_limit,
            contract_abi,
            chain_id,
            net,
        }
    }
}
