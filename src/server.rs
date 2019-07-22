use clap::ArgMatches;

pub fn run(logger: slog::Logger, arg: &ArgMatches) -> Result<(), String> {
    info!(logger, "server binds to {}", arg.value_of("bind").unwrap());
    Ok(())
}