use clap::ArgMatches;

#[derive(RustEmbed)]
#[folder = "src/contract/"]
struct Asset;


pub fn run(logger: slog::Logger, arg: &ArgMatches) -> Result<(), String> {
    let net = arg.value_of("net").unwrap();

    info!(logger, "deploy called to the {} network", net);

    let contract_abi = Asset::get("PriceOracle.abi").unwrap();
    info!(logger, "{:?}", std::str::from_utf8(contract_abi.as_ref()));

    Ok(())
}