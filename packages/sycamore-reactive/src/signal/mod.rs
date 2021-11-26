// Separate out the private implementation details from the public API. We need
// do this at the module level because we want to use traits, and this is one
// way to permit a public subtrait that depends on a private supertrait.

mod private;
mod public;

pub(crate) use crate::signal::private::*;
pub use crate::signal::public::*;
