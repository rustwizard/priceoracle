extern crate clap;
#[macro_use]
extern crate slog;
extern crate slog_term;
#[macro_use]
extern crate rust_embed;

use slog::Drain;

use clap::{App, Arg, ArgMatches, SubCommand};
use std::process;
mod server;
mod deploy;
mod updateprice;

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
        .subcommand(
            SubCommand::with_name("server")
                .about("starts http server")
                .arg(
                    Arg::with_name("bind")
                        .required(true)
                        .env("PO_SERVER_BIND")
                        .short("b")
                        .long("bind")
                        .help("address:port"),
                ),
        )
        .subcommand(
            SubCommand::with_name("deploy")
                .about("deploys contract to the ethereum")
                .arg(
                    Arg::with_name("net")
                        .required(true)
                        .env("PO_ETHEREUM_NETWORK")
                        .short("n")
                        .long("net")
                        .help("mainnet or testnet"),
                )
                .arg(
                    Arg::with_name("from_addr")
                        .env("PO_ETHEREUM_FROM_ADDR")
                        .long("from_addr")
                        .help("address will be used for contract deploy"),
                )
                .arg(
                    Arg::with_name("private_key")
                        .env("PO_ETHEREUM_PRIVATE_KEY")
                        .long("private_key")
                        .help("private key for tx signing"),
                ),
        )
        .subcommand(
            SubCommand::with_name("updateprice")
                .about("update price in the contract")
                .arg(
                    Arg::with_name("net")
                        .required(true)
                        .env("PO_ETHEREUM_NETWORK")
                        .long("net")
                        .help("mainnet or testnet"),
                )
                .arg(
                    Arg::with_name("newprice")
                        .required(true)
                        .takes_value(true)
                        .short("np")
                        .long("newprice")
                        .help("set new price in uint256"),
                )
                .arg(
                    Arg::with_name("contractaddr")
                        .required(true)
                        .env("PO_CONTRACT_ADDRESS")
                        .short("ca")
                        .long("contractaddr")
                        .help("address of the contract int the Ethereum network"),
                ),

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
        ("server", Some(server_matches)) => server::run(logger, server_matches),
        ("deploy", Some(deploy_matches)) => deploy::run(logger, deploy_matches),
        ("updateprice", Some(up_matches)) => updateprice::run(logger, up_matches),
        ("", None) => {
            error!(logger, "no subcommand was used");
            Ok(())
        }
        _ => unreachable!(),
    }
}
