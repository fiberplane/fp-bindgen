mod example_protocol;

use example_protocol::*;
use std::collections::BTreeMap;

// I would have preferred to use an `#[fp_export]` annotation here, rather than this rules macro.
// It would look more pleasing IMO, and would give us the ability to provide better error messages
// in case signatures don't match. Unfortunately, procedural macros must be in their own crate,
// and generating an entire crate just for this would introduce a lot of complexity.
fp_export!(
    fn my_plain_exported_function(a: u32, b: u32) -> u32 {
        a + my_plain_imported_function(a, b)
    }
);

fp_export!(
    fn my_complex_exported_function(a: ComplexHostToGuest) -> ComplexGuestToHost {
        let simple = Simple {
            bar: "bar".to_owned(),
            foo: 1,
        };

        my_complex_imported_function(ComplexGuestToHost {
            map: BTreeMap::new(),
            simple: simple.clone(),
        });

        ComplexGuestToHost {
            map: BTreeMap::new(),
            simple,
        }
    }
);

fp_export!(
    async fn my_async_exported_function() -> ComplexGuestToHost {
        let result = my_async_imported_function().await;
        ComplexGuestToHost {
            map: BTreeMap::new(),
            simple: result.simple,
        }
    }
);
