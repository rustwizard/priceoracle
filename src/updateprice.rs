use clap::ArgMatches;

pub fn run(logger: slog::Logger, arg: &ArgMatches) -> Result<(), String> {
    let net = arg.value_of("net").unwrap();
    let newprice = arg.value_of("newprice").unwrap();
    let ca = arg.value_of("contractaddr").unwrap();
    info!(logger, "updateprice called to the {} network with {} price. contractaddr {}",
          net, newprice, ca);
    Ok(())
}