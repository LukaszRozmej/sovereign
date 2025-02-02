use sov_modules_api::{Context, Spec};
use sov_state::WorkingSet;

/// Represents a transaction after verification.
pub trait VerifiedTx {
    type Address;
    fn sender(&self) -> &Self::Address;
    fn runtime_message(&self) -> &[u8];
}

/// TxHooks allows injecting custom logic into a transaction processing pipeline.
pub trait TxHooks {
    type Context: Context;
    type Transaction;
    type VerifiedTx: VerifiedTx<Address = <Self::Context as Spec>::Address>;

    /// pre_dispatch_tx_hook runs just before a transaction is dispatched to an appropriate module.
    fn pre_dispatch_tx_hook(
        &mut self,
        tx: Self::Transaction,
        working_set: &mut WorkingSet<<Self::Context as Spec>::Storage>,
    ) -> anyhow::Result<Self::VerifiedTx>;

    /// post_dispatch_tx_hook runs after the tx is dispatched to an appropriate module.
    fn post_dispatch_tx_hook(
        &mut self,
        tx: Self::VerifiedTx,
        working_set: &mut WorkingSet<<Self::Context as Spec>::Storage>,
    );
}
