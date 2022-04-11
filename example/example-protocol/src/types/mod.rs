mod aliases;
pub use aliases::*;

mod dead_code;
pub use dead_code::*;

mod flattening;
pub use flattening::*;

mod generics;
pub use generics::*;

mod http;
pub use self::http::*;

mod inline_docs;
pub use inline_docs::*;

mod renaming;
pub use renaming::*;

mod tagged_enums;
pub use tagged_enums::*;
