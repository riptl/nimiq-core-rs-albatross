use std::sync::Arc;

use futures::{
    channel::mpsc,
    stream::StreamExt,
    sink::SinkExt,
};

use nimiq_blockchain_albatross::Blockchain;
use nimiq_block_production_albatross::{test_utils::*, BlockProducer};
use nimiq_primitives::networks::NetworkId;
use nimiq_database::volatile::VolatileEnvironment;
use nimiq_consensus_albatross::sync::block_queue::{BlockQueue, PeerTrackingAndRequestComponent, BlockTopic};
use nimiq_network_mock::{MockHub, MockNetwork};


#[tokio::test]
async fn test_no_missing_blocks() {
    let hub = MockHub::default();

    let env = VolatileEnvironment::new(10).unwrap();
    let blockchain = Arc::new(Blockchain::new(env.clone(), NetworkId::UnitAlbatross).unwrap());
    let request_component = PeerTrackingAndRequestComponent::default();
    let producer = BlockProducer::new(Arc::clone(&blockchain1), Arc::clone(&mempool1), keypair.clone());
    let (tx, rx) = mpsc::channel(4);

    // push one micro block to the queue
    tx.send(producer.next_micro_block(blockchain.time.now(), 0, None, vec![], vec![0x42])).await;

    let block_queue = BlockQueue::new(
        Default::default(),
        blockchain,
        request_component,
        rx.boxed(),
    );
}
