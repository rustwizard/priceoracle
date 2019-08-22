use clap::ArgMatches;
use web3::contract::{Contract, Options};
use web3::futures::Future;
use web3::types::{U256, Address, H256};
use std::time::Duration;

use web3::Transport;

#[derive(RustEmbed)]
#[folder = "src/contract/"]
struct Asset;

pub fn run_with_http(logger: slog::Logger, arg: &ArgMatches) -> Result<(), String> {
    let config = Config::new(arg);
    info!(logger, "deploy called to the {} network with {:?}", config.net, config.from_addr);


    let (eloop,ethan) = web3::transports::Http::new(&config.net).unwrap();
    eloop.into_remote();

    let web3 = web3::Web3::new(ethan);

    let contract_address =  match config.from_addr {
        None => with_own_eth_node(web3, &logger, config) ,
        Some(_) => with_existing_wallet(web3, &logger, config),
    };

    info!(logger,"contract address: {:?}", contract_address);

    Ok(())
}

pub fn run_with_ws(logger: slog::Logger, arg: &ArgMatches) -> Result<(), String> {
    let config = Config::new(arg);
    info!(logger, "deploy called to the {:?} network with {:?}", config.net, config.from_addr);


    let (eloop,ethan) = web3::transports::WebSocket::new(&config.net).unwrap();
    eloop.into_remote();

    let web3 = web3::Web3::new(ethan);

    let contract_address =  match config.from_addr {
        None => with_own_eth_node(web3, &logger, config) ,
        Some(_) => with_existing_wallet(web3, &logger, config),
    };

    info!(logger,"contract address: {:?}", contract_address.unwrap());

    Ok(())
}

fn with_existing_wallet(eth_client: web3::Web3<impl Transport>,
                        logger: &slog::Logger,
                        conf: Config) -> Result<(Address), String> {

    let gas_price = match eth_client.eth().gas_price().wait() {
        Ok(gas_price) => gas_price,
        Err(e) => return Err(e.to_string()),
    };

    info!(logger,"deploy contract from {:?} with suggested gas_price: {:?}",
          conf.from_addr, gas_price);


    let nonce_cnt  = match eth_client.eth().transaction_count(conf.from_addr.unwrap(), None).wait() {
        Ok(nonce) => nonce,
        Err(e) => return Err(e.to_string()),
    };

    let tx_request = ethtxsign::RawTransaction {
        to: None,
        gas: conf.gas_limit,
        gas_price: gas_price.into(),
        value: 0.into(),
        data: conf.contract_bytecode,
        nonce: nonce_cnt,
    };

    let tx = tx_request.sign(&conf.pvt_key, &conf.chain_id);

    let result =
        eth_client.send_raw_transaction_with_confirmation(tx.into(),Duration::from_secs(1), 1);

    let receipt = match result.wait() {
        Ok(receipt) => receipt,
        Err(e) => return Err(e.to_string()),
    };

    info!(logger, "tx {} created", receipt.transaction_hash);

    Ok(receipt.contract_address.unwrap())
}

fn with_own_eth_node(eth_client: web3::Web3<impl Transport>, logger: &slog::Logger, conf: Config) -> Result<(Address), String> {
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
            opt.gas = Some(conf.gas_limit);
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

struct Config {
    from_addr: Option<Address>,
    pvt_key: H256,
    gas_limit: U256,
    contract_bytecode: Vec<u8>,
    chain_id: u8,
    net: String
}

impl Config {
    fn new(arg: &ArgMatches) -> Self {
        let net = arg.value_of("net").unwrap().to_string();

        let my_account =  arg.value_of("from_addr").unwrap();
        let fr: Address = my_account.parse().unwrap();

        let pk = arg.value_of("private_key").unwrap();
        let pvt_key = ethtxsign::pvt_key_from_slice(hex::decode(pk.as_bytes()).unwrap().as_slice()).unwrap();

        let gl = arg.value_of("gas_limit").unwrap();
        let gas_limit: U256 = U256::from_dec_str(gl).unwrap();

        let cid = arg.value_of("chain_id").unwrap();
        let chain_id = cid.parse::<u8>().unwrap();

        let cb = Asset::get("PriceOracle.bin").unwrap();
        let contract_bytecode = hex::decode(cb.as_ref()).unwrap();

        Config{
            from_addr: Some(fr),
            pvt_key,
            gas_limit,
            contract_bytecode,
            chain_id,
            net
        }
    }
}