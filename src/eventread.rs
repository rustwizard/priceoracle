use crate::web3util;
use clap::ArgMatches;
use web3::types::{Address, FilterBuilder, BlockNumber};
use web3::futures::{Future, Stream};
extern crate tokio_core;

pub fn run_with_ws(logger: slog::Logger, arg: &ArgMatches) -> Result<(), String> {
    let config = Config::new(arg);
    let mut eloop = tokio_core::reactor::Core::new().unwrap();
    let web3 =
        web3::Web3::new(web3::transports::WebSocket::with_event_loop(&config.net, &eloop.handle()).unwrap());

    //let (eloop, ws) = web3::transports::WebSocket::new(&config.net).unwrap();

    let topic = web3util::h256_topic(ethtxsign::keccak256_hash("PriceChanged(uint256)".as_ref()));



    info!(
        logger,
        "readevent runs on the {} network with contractaddr {} with topic {} from blocknum {}",
        config.net,
        config.contract_addr.unwrap(),
        topic.unwrap(),
        config.block_num
    );

        let filter = FilterBuilder::default()
            .address(vec![config.contract_addr.unwrap()])
            .topics(Some(vec![topic.unwrap()]), None, None, None)
            .from_block(BlockNumber::from(config.block_num))
            .to_block(BlockNumber::Latest)
            .build();

    info!(logger, "filter {:?}", filter);

    eloop.run(web3.eth_subscribe()
            .subscribe_logs(filter)
            .then(|sub| {
                sub.unwrap().for_each(|log| {
                    info!(logger, "got event: {:?}", log);
                    Ok(())
                })
            })

    ).unwrap();

    Ok(())
}

struct Config {
    contract_addr: Option<Address>,
    net: String,
    block_num: u64,
}

impl Config {
    fn new(arg: &ArgMatches) -> Self {
        let net = arg.value_of("net").unwrap().to_string();


        let ca = arg.value_of("contractaddr").unwrap();
        let contract_address: Address = ca.parse().unwrap();

        let block_num: u64 = arg.value_of("blocknum").unwrap().parse().unwrap();

        Config {
            contract_addr: Some(contract_address),
            net,
            block_num,
        }
    }
}
