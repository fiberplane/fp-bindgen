use example_bindings::*;
use once_cell::sync::Lazy;
use redux_example::{ReduxAction, ReduxState, StateUpdate};
use std::{cell::RefCell, rc::Rc};

// We maintain the global state in a mutable static so that we do not need to pass it from
// JavaScript every time we call the reducer. This avoids significant serialization overhead we
// would incur otherwise.
static mut STATE: Lazy<RefCell<ReduxState>> = Lazy::new(|| RefCell::new(ReduxState::default()));

#[fp_export_impl(example_bindings)]
fn reducer_bridge(action: ReduxAction) -> StateUpdate {
    // Accessing a global static instance is unsafe, because it could cause data
    // races. This should not be a problem here as long as we only call this
    // function from WASM in a single-threaded context:
    let old_state = unsafe { STATE.get_mut() };
    let new_state = reducer(old_state, action);

    let state_update = StateUpdate::from_state(old_state, &new_state);

    unsafe {
        STATE.replace(new_state);
    }

    state_update
}

fn reducer(state: &ReduxState, action: ReduxAction) -> ReduxState {
    let mut state = state.clone();
    match action {
        ReduxAction::ClearTitle => {
            state.title = Rc::new(String::default());
            state.revision += 1;
        }
        ReduxAction::UpdateTitle { title } => {
            state.title = Rc::new(title);
            state.revision += 1;
        }
    }

    state
}
