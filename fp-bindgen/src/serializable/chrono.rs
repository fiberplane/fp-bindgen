use super::Serializable;
use crate::types::{CustomType, Type};
use std::collections::{BTreeMap, BTreeSet};

impl Serializable for chrono::Utc {
    fn name() -> String {
        "chrono::Utc".to_owned()
    }

    fn ty() -> Type {
        Type::Custom(CustomType {
            name: "Utc".to_owned(),
            type_args: vec![],
            rs_ty: "chrono::Utc".to_owned(),
            rs_dependencies: BTreeMap::from([(
                "chrono".to_owned(),
                r#"{ version = "0.4", features = ["serde"] }"#.to_owned(),
            )]),
            ts_ty: "UNIMPLEMENTED".to_owned(), // *should* never appear in the generated output
        })
    }

    fn build_dependencies() -> BTreeSet<Type> {
        BTreeSet::new()
    }
}

impl<T> Serializable for chrono::DateTime<T>
where
    T: chrono::TimeZone + Serializable,
{
    fn name() -> String {
        "chrono::DateTime<T>".to_owned()
    }

    fn ty() -> Type {
        Type::Custom(CustomType {
            name: "DateTime".to_owned(),
            type_args: vec![],
            rs_ty: format!("chrono::DateTime<{}>", T::name()),
            rs_dependencies: BTreeMap::from([(
                "chrono".to_owned(),
                r#"{ version = "0.4", features = ["serde"] }"#.to_owned(),
            )]),
            ts_ty: "string".to_owned(),
        })
    }

    fn build_dependencies() -> BTreeSet<Type> {
        T::type_with_dependencies()
    }
}
