mod modules;

use modules::{first_test_module, second_test_module};
use sov_modules_api::mocks::MockContext;
use sov_modules_api::{Context, Module, ModuleInfo};
use sov_modules_macros::{DispatchCall, DispatchQuery, Genesis, MessageCodec};
use sov_state::ProverStorage;

// Debugging hint: To expand the macro in tests run: `cargo expand --test tests`
#[derive(Genesis, DispatchQuery, DispatchCall, MessageCodec)]
struct Runtime<C>
where
    C: Context,
{
    first: first_test_module::FirstTestStruct<C>,
    second: second_test_module::SecondTestStruct<C>,
}

impl<C: Context> Runtime<C> {
    fn new() -> Self {
        Self {
            first: first_test_module::FirstTestStruct::<C>::new(),
            second: second_test_module::SecondTestStruct::<C>::new(),
        }
    }
}

fn main() {
    use sov_modules_api::{DispatchQuery, Genesis};

    type C = MockContext;
    let storage = ProverStorage::temporary();
    let working_set = &mut sov_state::WorkingSet::new(storage);
    let runtime = &mut Runtime::<C>::new();
    runtime.genesis(working_set).unwrap();

    {
        let message = RuntimeQuery::<C>::first(());
        let response = runtime.dispatch_query(message, working_set);
        assert_eq!(response.response, vec![1]);
    }

    {
        let message = RuntimeQuery::<C>::second(second_test_module::TestType {});
        let response = runtime.dispatch_query(message, working_set);
        assert_eq!(response.response, vec![2]);
    }
}
