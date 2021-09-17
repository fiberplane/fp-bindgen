mod r#async;
pub mod functions;
mod queue;
mod support;
mod task;
pub mod types;

pub mod __fp_macro {
    pub use super::r#async::{AsyncValue, __fp_guest_resolve_async_value};
    pub use super::support::{
        FatPtr, __fp_free, __fp_malloc, export_value_to_host, from_fat_ptr, import_value_from_host,
        malloc, to_fat_ptr,
    };
    pub use super::task::Task;
}

pub mod prelude {
    pub use super::__fp_macro;
    pub use super::functions::*;
    pub use super::types::*;
}
