use clap::ArgMatches;

pub fn run(logger: slog::Logger, arg: &ArgMatches) -> Result<(), String> {
    let net = arg.value_of("net").unwrap();

    info!(logger, "deploy called to the {} network", net);

    Ok(())
}