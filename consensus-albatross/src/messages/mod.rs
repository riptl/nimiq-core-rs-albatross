pub use self::request_response::RequestResponseMessage;
use crate::subscription::Subscription;
use beserial::{Deserialize, Serialize};
use block_albatross::{BlockComponents, MacroBlock};
use hash::Blake2bHash;
use network_interface::message::*;
use transaction::Transaction;

mod request_response;

/*
The consensus module uses the following messages:
200 RequestResponseMessage<RequestBlockHashes>
201 RequestResponseMessage<BlockHashes>
202 RequestResponseMessage<RequestEpoch>
203 RequestResponseMessage<Epoch>
204 Subscription
205 BlockAnnouncement
*/

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum Object<T: Serialize + Deserialize> {
    #[beserial(discriminant = 0)]
    Hash(Blake2bHash),
    #[beserial(discriminant = 1)]
    Object(T),
}

impl<T: Serialize + Deserialize> Object<T> {
    pub fn with_object(object: T) -> Self {
        Object::Object(object)
    }

    pub fn with_hash(hash: Blake2bHash) -> Self {
        Object::Hash(hash)
    }

    pub fn is_hash(&self) -> bool {
        if let Object::Hash(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_object(&self) -> bool {
        !self.is_hash()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum Objects<T: Serialize + Deserialize> {
    #[beserial(discriminant = 0)]
    Hashes(#[beserial(len_type(u16))] Vec<Blake2bHash>),
    #[beserial(discriminant = 1)]
    Objects(#[beserial(len_type(u16))] Vec<T>),
}

impl<T: Serialize + Deserialize> Objects<T> {
    pub const MAX_HASHES: usize = 1000;
    pub const MAX_OBJECTS: usize = 1000;

    pub fn with_objects(objects: Vec<T>) -> Self {
        Objects::Objects(objects)
    }

    pub fn with_hashes(hashes: Vec<Blake2bHash>) -> Self {
        Objects::Hashes(hashes)
    }

    pub fn contains_hashes(&self) -> bool {
        if let Objects::Hashes(_) = self {
            true
        } else {
            false
        }
    }

    pub fn contains_objects(&self) -> bool {
        !self.contains_hashes()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockHashes {
    #[beserial(len_type(u16))]
    pub hashes: Vec<Blake2bHash>,
}

impl Message for RequestResponseMessage<BlockHashes> {
    const TYPE_ID: u64 = 201;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[repr(u8)]
pub enum RequestBlockHashesFilter {
    All = 1,
    ElectionOnly = 2,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestBlockHashes {
    #[beserial(len_type(u16, limit = 128))]
    pub locators: Vec<Blake2bHash>,
    pub max_blocks: u16,
    pub filter: RequestBlockHashesFilter,
}

impl Message for RequestResponseMessage<RequestBlockHashes> {
    const TYPE_ID: u64 = 200;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestEpoch {
    pub hash: Blake2bHash,
}

impl Message for RequestResponseMessage<RequestEpoch> {
    const TYPE_ID: u64 = 202;
}

// TODO: Syncing all of this from one peer is quite a lot
//  and it is probably also inefficient to sync all transactions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Epoch {
    pub block: MacroBlock,
    #[beserial(len_type(u32))]
    pub transactions: Vec<Transaction>,
}

impl Message for RequestResponseMessage<Epoch> {
    const TYPE_ID: u64 = 203;
}

impl Message for Subscription {
    const TYPE_ID: u64 = 204;
}

pub type BlockAnnouncement = Object<BlockComponents>;

impl Message for BlockAnnouncement {
    const TYPE_ID: u64 = 205;
}
