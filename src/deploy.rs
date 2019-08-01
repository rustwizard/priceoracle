use clap::ArgMatches;
use web3::contract::{Contract, Options};
use web3::futures::Future;
use web3::types::{U256, Address};
use web3::transports::Http;

#[derive(RustEmbed)]
#[folder = "src/contract/"]
struct Asset;


pub fn run(logger: slog::Logger, arg: &ArgMatches) -> Result<(), String> {
    let net = arg.value_of("net").unwrap();

    info!(logger, "deploy called to the {} network", net);

    let (eloop, http) = web3::transports::Http::new(net).unwrap();
    eloop.into_remote();

    let web3 = web3::Web3::new(http);
    let contract_address =  match with_eth_node(web3, &logger) {
        Err(e) => return Err(e.to_string()),
        Ok(a) => a,
    };

    info!(logger,"contract address: {:?}", contract_address);

    Ok(())
}

fn with_eth_node(eth_client: web3::Web3<Http>, logger: &slog::Logger) -> Result<(Address), String> {
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