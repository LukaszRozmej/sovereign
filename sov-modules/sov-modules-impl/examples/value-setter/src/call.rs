use anyhow::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use sov_modules_api::CallResponse;
use sov_state::WorkingSet;
use std::fmt::Debug;
use thiserror::Error;

use super::ValueSetter;

#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq)]
pub struct SetValue {
    pub new_value: u32,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq)]
pub enum CallMessage {
    DoSetValue(SetValue),
}

#[derive(Debug, Error)]
enum SetValueError {
    #[error("Only admin can change the value")]
    WrongSender,
}

impl<C: sov_modules_api::Context> ValueSetter<C> {
    /// Sets `value` field to the `new_value`, only admin is authorized to call this method.
    pub(crate) fn set_value(
        &self,
        new_value: u32,
        context: &C,
        working_set: &mut WorkingSet<C::Storage>,
    ) -> Result<sov_modules_api::CallResponse> {
        let mut response = CallResponse::default();

        let admin = self.admin.get_or_err(working_set)?;

        if &admin != context.sender() {
            // Here we use a custom error type.
            Err(SetValueError::WrongSender)?;
        }

        self.value.set(new_value, working_set);
        response.add_event("set", &format!("value_set: {new_value:?}"));

        Ok(response)
    }
}
