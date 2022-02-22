use super::Serializable;

#[cfg(feature = "rmpv-compat")]
impl Serializable for rmpv::Value {
    fn name() -> String {
        "Value".to_owned()
    }

    fn ty() -> Type {
        Type::Custom(CustomType {
            name: "Value".to_owned(),
            type_args: vec![],
            rs_ty: "rmpv::Value".to_owned(),
            rs_dependencies: BTreeMap::from([(
                "rmpv".to_owned(),
                r#"{ version = "1.0.0", features = ["with-serde"] }"#.to_owned(),
            )]),
            ts_ty: "string".to_owned(),
        })
    }

    fn build_dependencies() -> BTreeSet<Type> {
        BTreeSet::new()
    }
}
