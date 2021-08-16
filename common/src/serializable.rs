use crate::Type;

pub trait Serializable {
    /// The name of the type as defined in the protocol.
    fn name() -> String;

    /// The data structure item.
    fn item() -> Type;

    /// Whether this type is a primitive.
    fn is_primitive() -> bool;

    /// Other (non-primitive) data structures this data structure depends on.
    fn dependencies() -> Vec<Type>;
}
