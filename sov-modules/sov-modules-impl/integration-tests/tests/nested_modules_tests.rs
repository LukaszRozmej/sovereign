use sov_modules_api::mocks::{MockContext, ZkMockContext};
use sov_modules_api::{Context, ModuleInfo, Prefix};
use sov_modules_macros::ModuleInfo;
use sov_state::storage::{StorageKey, StorageValue};
use sov_state::{ProverStorage, StateMap, StateValue, Storage, WorkingSet, ZkStorage};

pub mod module_a {
    use super::*;

    #[derive(ModuleInfo)]
    pub(crate) struct ModuleA<C: Context> {
        #[address]
        pub address_module_a: C::Address,

        #[state]
        pub(crate) state_1_a: StateMap<String, String>,

        #[state]
        pub(crate) state_2_a: StateValue<String>,
    }

    impl<C: Context> ModuleA<C> {
        pub fn update(&mut self, key: &str, value: &str, working_set: &mut WorkingSet<C::Storage>) {
            self.state_1_a
                .set(&key.to_owned(), value.to_owned(), working_set);
            self.state_2_a.set(value.to_owned(), working_set)
        }
    }
}

pub mod module_b {
    use super::*;

    #[derive(ModuleInfo)]
    pub(crate) struct ModuleB<C: Context> {
        #[address]
        pub address_module_b: C::Address,

        #[state]
        state_1_b: StateMap<String, String>,

        #[module]
        pub(crate) mod_1_a: module_a::ModuleA<C>,
    }

    impl<C: Context> ModuleB<C> {
        pub fn update(&mut self, key: &str, value: &str, working_set: &mut WorkingSet<C::Storage>) {
            self.state_1_b
                .set(&key.to_owned(), value.to_owned(), working_set);
            self.mod_1_a.update("key_from_b", value, working_set);
        }
    }
}

mod module_c {
    use super::*;

    #[derive(ModuleInfo)]
    pub(crate) struct ModuleC<C: Context> {
        #[address]
        pub address: C::Address,

        #[module]
        pub(crate) mod_1_a: module_a::ModuleA<C>,

        #[module]
        mod_1_b: module_b::ModuleB<C>,
    }

    impl<C: Context> ModuleC<C> {
        pub fn execute(
            &mut self,
            key: &str,
            value: &str,
            working_set: &mut WorkingSet<C::Storage>,
        ) {
            self.mod_1_a.update(key, value, working_set);
            self.mod_1_b.update(key, value, working_set);
            self.mod_1_a.update(key, value, working_set);
        }
    }
}

#[test]
fn nested_module_call_test() {
    let native_storage = ProverStorage::temporary();
    let working_set = &mut WorkingSet::new(native_storage.clone());

    // Test the `native` execution.
    {
        execute_module_logic::<MockContext>(working_set);
        test_state_update::<MockContext>(working_set);
    }
    let (log, witness) = working_set.freeze();
    native_storage
        .validate_and_commit(log, &witness)
        .expect("State update is valid");

    // Test the `zk` execution.
    {
        let zk_storage = ZkStorage::new([0u8; 32]);
        let working_set = &mut WorkingSet::with_witness(zk_storage, witness);
        execute_module_logic::<ZkMockContext>(working_set);
        test_state_update::<ZkMockContext>(working_set);
    }
}

fn execute_module_logic<C: Context>(working_set: &mut WorkingSet<C::Storage>) {
    let module = &mut module_c::ModuleC::<C>::new();
    module.execute("some_key", "some_value", working_set);
}

fn test_state_update<C: Context>(working_set: &mut WorkingSet<C::Storage>) {
    let module = <module_c::ModuleC<C> as ModuleInfo>::new();

    let expected_value = StorageValue::new("some_value");

    {
        let prefix = Prefix::new_storage("nested_modules_tests::module_a", "ModuleA", "state_1_a");
        let key = StorageKey::new(&prefix.into(), &"some_key");
        let value = working_set.get(key).unwrap();

        assert_eq!(expected_value, value);
    }

    {
        let prefix = Prefix::new_storage("nested_modules_tests::module_b", "ModuleB", "state_1_b");
        let key = StorageKey::new(&prefix.into(), &"some_key");
        let value = working_set.get(key).unwrap();

        assert_eq!(expected_value, value);
    }

    {
        let prefix = Prefix::new_storage("nested_modules_tests::module_a", "ModuleA", "state_1_a");
        let key = StorageKey::new(&prefix.into(), &"key_from_b");
        let value = working_set.get(key).unwrap();

        assert_eq!(expected_value, value);
    }

    {
        let value = module.mod_1_a.state_2_a.get(working_set).unwrap();
        assert_eq!("some_value".to_owned(), value);
    }
}
