use std::sync::Arc;

use futures::{
    channel::mpsc,
    stream::StreamExt,
    sink::SinkExt,
};

use beserial::Deserialize;
use nimiq_bls::{KeyPair, SecretKey};
use nimiq_block_albatross::Block;
use nimiq_blockchain_albatross::Blockchain;
use nimiq_block_production_albatross::BlockProducer;
use nimiq_mempool::{Mempool, MempoolConfig};
use nimiq_primitives::networks::NetworkId;
use nimiq_database::volatile::VolatileEnvironment;
use nimiq_consensus_albatross::sync::block_queue::{BlockQueue, PeerTrackingAndRequestComponent};


/// Secret key of validator. Tests run with `network-primitives/src/genesis/unit-albatross.toml`
const SECRET_KEY: &str =
    "196ffdb1a8acc7cbd76a251aeac0600a1d68b3aba1eba823b5e4dc5dbdcdc730afa752c05ab4f6ef8518384ad514f403c5a088a22b17bf1bc14f8ff8decc2a512c0a200f68d7bdf5a319b30356fe8d1d75ef510aed7a8660968c216c328a0000";


#[tokio::test]
async fn send_single_micro_block_to_block_queue() {
    let keypair = KeyPair::from(SecretKey::deserialize_from_vec(&hex::decode(SECRET_KEY).unwrap()).unwrap());
    let env = VolatileEnvironment::new(10).unwrap();
    let blockchain = Arc::new(Blockchain::new(env, NetworkId::UnitAlbatross).unwrap());
    let mempool = Mempool::new(Arc::clone(&blockchain), MempoolConfig::default());
    let producer = BlockProducer::new(Arc::clone(&blockchain), Arc::clone(&mempool), keypair);
    let request_component = PeerTrackingAndRequestComponent::default();
    let (mut tx, rx) = mpsc::channel(32);

    let mut block_queue = BlockQueue::new(
        Default::default(),
        Arc::clone(&blockchain),
        request_component,
        rx.boxed(),
    );

    // push one micro block to the queue
    let block = Block::Micro(producer.next_micro_block(blockchain.time.now(), 0, None, vec![], vec![0x42]));
    tx.send(block).await.unwrap();

    assert_eq!(blockchain.block_number(), 0);

    // run the block_queue one iteration, i.e. until it processed one block
    block_queue.next().await;

    // The produced block is without gap and should go right into the blockchain
    assert_eq!(blockchain.block_number(), 1);
    assert!(block_queue.buffered_blocks().collect::<Vec<_>>().is_empty());
}

#[tokio::test]
async fn send_two_micro_blocks_out_of_order() {
    let keypair = KeyPair::from(SecretKey::deserialize_from_vec(&hex::decode(SECRET_KEY).unwrap()).unwrap());
    let env1 = VolatileEnvironment::new(10).unwrap();
    let env2 = VolatileEnvironment::new(10).unwrap();
    let blockchain1 = Arc::new(Blockchain::new(env1, NetworkId::UnitAlbatross).unwrap());
    let blockchain2 = Arc::new(Blockchain::new(env2, NetworkId::UnitAlbatross).unwrap());
    let mempool = Mempool::new(Arc::clone(&blockchain2), MempoolConfig::default());
    let producer = BlockProducer::new(Arc::clone(&blockchain2), Arc::clone(&mempool), keypair);
    let request_component = PeerTrackingAndRequestComponent::default();
    let (mut tx, rx) = mpsc::channel(32);

    let mut block_queue = BlockQueue::new(
        Default::default(),
        Arc::clone(&blockchain1),
        request_component,
        rx.boxed(),
    );

    let block1 = Block::Micro(producer.next_micro_block(blockchain2.time.now(), 0, None, vec![], vec![0x42]));
    blockchain2.push(block1.clone()).unwrap(); // push it, so the producer actually produces a block at height 2
    let block2 = Block::Micro(producer.next_micro_block(blockchain2.time.now() + 1000, 0, None, vec![], vec![0x42]));

    // send block2 first
    tx.send(block2.clone()).await.unwrap();

    assert_eq!(blockchain1.block_number(), 0);

    // run the block_queue one iteration, i.e. until it processed one block
    block_queue.next().await;

    // this block should be buffered now
    assert_eq!(blockchain1.block_number(), 0);
    let blocks = block_queue.buffered_blocks().collect::<Vec<_>>();
    assert_eq!(blocks.len(), 1);
    let (block_number, blocks) = blocks.get(0).unwrap();
    assert_eq!(*block_number, 2);
    assert_eq!(blocks[0], block2);

    // now send block1 to fill the gap
    tx.send(block1.clone()).await.unwrap();

    // run the block_queue one iteration, i.e. until it processed one block
    block_queue.next().await;

    // now both blocks should've been pushed to the blockchain
    assert_eq!(blockchain1.block_number(), 2);
    assert!(block_queue.buffered_blocks().collect::<Vec<_>>().is_empty());
    assert_eq!(blockchain1.get_block_at(1, true).unwrap(), block1);
    assert_eq!(blockchain1.get_block_at(2, true).unwrap(), block2);
}