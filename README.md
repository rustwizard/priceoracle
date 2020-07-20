# priceoracle
Simple implementation of Ethereum Price Oracle Contract DAPP.

The service uses https://min-api.cryptocompare.com/ for getting price for the ETH-BTC pair and then update
the price value in the contract at Ethereum network

Before run you should deploy PriceOracle contract to the Ethereum network with `./priceoracle deploy` command 
or with Remix IDE(https://remix.ethereum.org/)

To compile and run project just do: `docker-compose build && docker-compose up`
