use fp_bindgen::prelude::Serializable;

// Types are included in the protocol simply by referencing them from function
// signatures or other types. Types that are not referenced anywhere will not
// be included in the protocol, even if they implement the `Serializable` trait.
//
// If you want to include a type in your protocol, even though no function or
// other type depends on it, you can include it explicitly by including a `use`
// statement inside either the `fp_import!` or the `fp_export!` macro.

/// This struct will not show up in the generated bindings, because no function
/// or data structures references it.
#[derive(Serializable)]
pub struct DeadCode {
    pub you_wont_see_this: bool,
}

/// This struct is also not referenced by any function or data structure, but
/// it will show up because there is an explicit `use` statement for it in the
/// `fp_import!` macro.
#[derive(Serializable)]
pub struct ExplicitedlyImportedType {
    pub you_will_see_this: bool,
}

/// Nested modules are supported in `use` statements.
pub mod submodule {
    use fp_bindgen::prelude::Serializable;

    pub mod nested {
        use fp_bindgen::prelude::Serializable;

        #[derive(Serializable)]
        pub struct GroupImportedType1 {
            pub you_will_see_this: bool,
        }
    }

    #[derive(Serializable)]
    pub struct GroupImportedType2 {
        pub you_will_see_this: bool,
    }
}
