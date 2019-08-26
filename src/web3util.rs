use web3::futures::Future;
use web3::types::{Address, U256};
use web3::Transport;

pub fn nonce(addr: Address, eth_client: &web3::Web3<impl Transport>) -> Result<U256, String> {
    let nonce_cnt = match eth_client.eth().transaction_count(addr, None).wait() {
        Ok(nonce) => nonce,
        Err(e) => return Err(e.to_string()),
    };
    Ok(nonce_cnt)
}
