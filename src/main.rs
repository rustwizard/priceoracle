extern crate clap;
#[macro_use]
extern crate slog;
extern crate slog_term;
#[macro_use]
extern crate rust_embed;
extern crate hex;

use slog::Drain;

use clap::{App, Arg, ArgMatches, SubCommand};
use std::process;

mod deploy;
mod server;
mod updateprice;
mod web3util;
mod eventread;

fn main() {
    let matches = build_app_get_matches();

    if let Err(e) = run(matches) {
        println!("Application error: {}", e);
        process::exit(1);
    }
}

fn run(matches: ArgMatches<'static>) -> Result<(), String> {
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
        ("deploy", Some(deploy_matches)) => {
            let transport = deploy_matches.value_of("transport").unwrap();
            if transport == "http" {
                deploy::run_with_http(logger, deploy_matches)
            } else {
                deploy::run_with_ws(logger, deploy_matches)
            }
        }
        ("updateprice", Some(up_matches)) => {
            let transport = up_matches.value_of("transport").unwrap();
            if transport == "http" {
                updateprice::run_with_http(logger, up_matches)
            } else {
                updateprice::run_with_ws(logger, up_matches)
            }
        }
        ("", None) => {
            error!(logger, "no subcommand was used");
            Ok(())
        }
        _ => unreachable!(),
    }
}

pub fn build_app_get_matches() -> ArgMatches<'static> {
    App::new("priceoracle")
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
                    Arg::with_name("transport")
                        .required(true)
                        .env("PO_ETHEREUM_TRANSPORT")
                        .long("transport")
                        .help("ws or http"),
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
                )
                .arg(
                    Arg::with_name("gas_limit")
                        .env("PO_ETHEREUM_GAS_LIMIT")
                        .long("gas_limit")
                        .help("gas limit for tx"),
                )
                .arg(
                    Arg::with_name("chain_id")
                        .env("PO_ETHEREUM_CHAIN_ID")
                        .long("chain_id")
                        .help("chain id for sign tx"),
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
                    Arg::with_name("transport")
                        .required(true)
                        .env("PO_ETHEREUM_TRANSPORT")
                        .long("transport")
                        .help("ws or http"),
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
                        .help("address of the contract in the Ethereum network"),
                )
                .arg(
                    Arg::with_name("gas_limit")
                        .env("PO_ETHEREUM_GAS_LIMIT")
                        .long("gas_limit")
                        .help("gas limit for tx"),
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
                )
                .arg(
                    Arg::with_name("chain_id")
                        .env("PO_ETHEREUM_CHAIN_ID")
                        .long("chain_id")
                        .help("chain id for sign tx"),
                ),
        ).subcommand(
        SubCommand::with_name("eventread")
            .about("read contract events")
            .arg(
                Arg::with_name("net")
                    .required(true)
                    .env("PO_ETHEREUM_NETWORK")
                    .long("net")
                    .help("mainnet or testnet"),
            )
            .arg(
                Arg::with_name("transport")
                    .required(true)
                    .env("PO_ETHEREUM_TRANSPORT")
                    .long("transport")
                    .help("ws or http"),
            )
            .arg(
                Arg::with_name("contractaddr")
                    .required(true)
                    .env("PO_CONTRACT_ADDRESS")
                    .short("ca")
                    .long("contractaddr")
                    .help("address of the contract in the Ethereum network"),
            )
            .arg(
            Arg::with_name("blocknum")
                .required(true)
                .env("PO_ETHEREUM_BLOCKNUM")
                .short("bn")
                .long("blocknum")
                .help(" blocknum from which we start parsing ethereum logs"),
            ),
    ).get_matches()
}
