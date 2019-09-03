use web3::futures::Future;
use web3::types::{Address, U256, H256};
use web3::Transport;

pub fn nonce(addr: Address, eth_client: &web3::Web3<impl Transport>) -> Result<U256, String> {
    let nonce_cnt = match eth_client.eth().transaction_count(addr, None).wait() {
        Ok(nonce) => nonce,
        Err(e) => return Err(e.to_string()),
    };
    Ok(nonce_cnt)
}

pub fn h256_topic(topic: Vec<u8>) -> Option<H256> {
    let mut h = H256::zero();
    h.as_bytes_mut().copy_from_slice(topic.as_slice());
    Some(h)
}