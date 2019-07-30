use clap::ArgMatches;
use web3::contract::{Contract, Options};
use web3::futures::Future;

#[derive(RustEmbed)]
#[folder = "src/contract/"]
struct Asset;


pub fn run(logger: slog::Logger, arg: &ArgMatches) -> Result<(), String> {
    let net = arg.value_of("net").unwrap();

    info!(logger, "deploy called to the {} network", net);

    let contract_abi = Asset::get("PriceOracle.abi").unwrap();
    info!(logger, "{:?}", std::str::from_utf8(contract_abi.as_ref()));

    let contract_bytecode = Asset::get("PriceOracle.bin").unwrap();

    let (eloop, http) = web3::transports::Http::new(net).unwrap();
    eloop.into_remote();

    let web3 = web3::Web3::new(http);

    let accounts = web3.eth().accounts().wait().unwrap();

    if accounts.len() == 0 {
        return Err(String::from("there is no any accounts for contract deploy"))
    }

    info!(logger,"Accounts: {:?}", accounts);

    let bc = std::str::from_utf8(contract_bytecode.as_ref()).unwrap();
    let contract = Contract::deploy(web3.eth(), contract_abi.as_ref())
        .unwrap()
        .confirmations(0)
        .options(Options::with(|opt| {
            opt.value = Some(0.into());
            opt.gas_price = Some(5.into());
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
    info!(logger,"contract address: {:?}", contract_address);

    Ok(())
}