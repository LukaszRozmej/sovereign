use crate::{Address, AddressTrait, Context, PublicKey, SigVerificationError, Signature, Spec};
use borsh::{BorshDeserialize, BorshSerialize};
use jmt::SimpleHasher;
use sov_state::ZkStorage;
use sov_state::{mocks::MockStorageSpec, ProverStorage};
use sovereign_sdk::core::types::ArrayWitness;
use std::convert::Infallible;

/// Mock for Spec::PublicKey, useful for testing.
#[derive(PartialEq, Eq, Clone, BorshDeserialize, BorshSerialize, Debug)]
pub struct MockPublicKey {
    pub_key: Vec<u8>,
}

impl MockPublicKey {
    pub fn new(pub_key: Vec<u8>) -> Self {
        Self { pub_key }
    }

    pub fn sign(&self, _msg: [u8; 32]) -> MockSignature {
        MockSignature { msg_sig: vec![] }
    }
}

impl TryFrom<&'static str> for MockPublicKey {
    type Error = Infallible;

    fn try_from(key: &'static str) -> Result<Self, Self::Error> {
        let key = key.as_bytes().to_vec();
        Ok(Self { pub_key: key })
    }
}

impl PublicKey for MockPublicKey {
    fn to_address<A: AddressTrait>(&self) -> A {
        let pub_key_hash = <MockContext as Spec>::Hasher::hash(&self.pub_key);
        A::try_from(&pub_key_hash).expect("todo")
    }
}

/// Mock for Spec::Signature, useful for testing.
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, PartialEq, Eq, Debug, Clone, Default)]
pub struct MockSignature {
    pub msg_sig: Vec<u8>,
}

impl Signature for MockSignature {
    type PublicKey = MockPublicKey;

    fn verify(
        &self,
        _pub_key: &Self::PublicKey,
        _msg_hash: [u8; 32],
    ) -> Result<(), SigVerificationError> {
        Ok(())
    }
}

/// Mock for Context, useful for testing.
#[derive(Clone, Debug, PartialEq)]
pub struct MockContext {
    pub sender: Address,
}

impl Spec for MockContext {
    type Address = Address;
    type Storage = ProverStorage<MockStorageSpec>;
    type Hasher = sha2::Sha256;
    type PublicKey = MockPublicKey;
    type Signature = MockSignature;
    type Witness = ArrayWitness;
}

impl Context for MockContext {
    fn sender(&self) -> &Self::Address {
        &self.sender
    }

    fn new(sender: Self::Address) -> Self {
        Self { sender }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ZkMockContext {
    pub sender: Address,
}

impl Spec for ZkMockContext {
    type Address = Address;
    type Storage = ZkStorage<MockStorageSpec>;
    type Hasher = sha2::Sha256;
    type PublicKey = MockPublicKey;
    type Signature = MockSignature;
    type Witness = ArrayWitness;
}

impl Context for ZkMockContext {
    fn sender(&self) -> &Self::Address {
        &self.sender
    }

    fn new(sender: Self::Address) -> Self {
        Self { sender }
    }
}
