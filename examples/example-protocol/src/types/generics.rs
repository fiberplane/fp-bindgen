use super::MyDateTime;
use fp_bindgen::prelude::Serializable;
use std::collections::BTreeMap;

// Generic arguments can be used, both on `std` types that take generic
// arguments as well as custom types with a `Serializable` implementation.

type FloatingPoint = Point<f64>;

/// A point of an arbitrary type.
#[derive(Serializable)]
pub struct Point<T> {
    pub value: T,
}

/// A point of an arbitrary type, with explicit trait bounds.
#[derive(Serializable)]
pub struct ExplicitBoundPoint<T: Serializable + std::fmt::Debug + std::fmt::Display> {
    pub value: T,
}

#[derive(Serializable)]
pub struct StructWithGenerics<T> {
    pub list: Vec<T>,
    pub points: Vec<Point<T>>,
    pub recursive: Vec<Point<Point<T>>>,
    pub complex_nested: Option<BTreeMap<String, Vec<FloatingPoint>>>,
    pub optional_timestamp: Option<MyDateTime>,
}
