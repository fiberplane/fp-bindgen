pub trait Deserializable {
    /// The name of the type in the target language.
    fn name() -> String;
}

pub trait Serializable {
    /// The name of the type in the target language.
    fn name() -> String;
}
