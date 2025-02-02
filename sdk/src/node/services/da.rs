use std::future::Future;

use crate::serial::{Decode, Encode};

// TODO: Rename to clarify distinction with DaApp
/// A DaService is the local side of an RPC connection talking to node of the DA layer
/// It is *not* part of the logic that is zk-proven. Rather, it provides functionality
/// to allow the sovereign SDK to interact with the DA layer's RPC network.
pub trait DaService {
    /// An L1 block, possibly excluding some irrelevant information
    type FilteredBlock: SlotData;
    type Future<T>: Future<Output = Result<T, Self::Error>>;
    // /// A transaction on the L1
    // type Transaction;
    // type Address;
    type Error: Send + Sync;

    /// Retrieve the data for the given height, waiting for it to be
    /// finalized if necessary. The block, once returned, must not be reverted
    /// without a consensus violation.
    fn get_finalized_at(&self, height: u64) -> Self::Future<Self::FilteredBlock>;

    /// Get the block at the given height, waiting for one to be mined if necessary.
    /// The returned block may not be final, and can be reverted without a consensus violation
    fn get_block_at(&self, height: u64) -> Self::Future<Self::FilteredBlock>;

    // TODO: Consider adding the send_transaction method
    // fn send_transaction(tx: Self::Transaction, sender: Self::Address)
}
pub trait SlotData: Encode + Decode + PartialEq + core::fmt::Debug + Clone {
    type BatchData;
    /// Encode any *non-batch* data (i.e. header, metadata, etc.) from this slot for storage. Batches contained
    /// in this slot are encoded and stored separately
    fn extra_data_for_storage(&self) -> Vec<u8>;
    fn reconstruct_from_storage(extra_data: &[u8], batches: Vec<Self::BatchData>) -> Self;
    fn hash(&self) -> [u8; 32];
}
