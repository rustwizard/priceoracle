extern crate clap;
#[macro_use]
extern crate slog;
extern crate slog_term;

use slog::Drain;

use clap::{App, Arg, ArgMatches, SubCommand};
use std::process;

fn main() {
    let matches = App::new("priceoracle")
        .version("0.0.1")
        .author("Rust Wizard")
        .about("eth2btc price oracle")
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .multiple(true)
                .help("verbosity level"),
        )
        .subcommand(SubCommand::with_name("server")
            .about("starts http server")
            .arg(Arg::with_name("bind")
                .required(true)
                .takes_value(true)
                .short("b")
                .long("bind")
                .help("address:port"))
        )
        .get_matches();
    if let Err(e) = run(matches) {
        println!("Application error: {}", e);
        process::exit(1);
    }
}

fn run(matches: ArgMatches) -> Result<(), String> {
    let min_log_level = match matches.occurrences_of("verbose") {
        0 => slog::Level::Info,
        1 => slog::Level::Debug,
        2 | _ => slog::Level::Trace,
    };

    let decorator = slog_term::TermDecorator::new().build();

    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog::LevelFilter(drain, min_log_level).fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let logger = slog::Logger::root(drain, o!());
    trace!(logger, "priceoracle_setup");
    // setting up app...
    debug!(logger, "load_configuration");
    trace!(logger, "priceoracle_setup_complete");
    // starting processing...
    info!(logger, "processing_started");

    match matches.subcommand() {
        ("server", Some(server_matches)) => {
            info!(logger, "server binds to {}", server_matches.value_of("bind").unwrap());
            Ok(())
        },
        ("", None) => {
            error!(logger, "no subcommand was used");
            Ok(())
        },
        _            => unreachable!(),
    }
}
