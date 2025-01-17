//! Drydoc
//!
//! Split up your documentation compontents into files
//!
//!```
#![doc = drydoc!(path = "docs/person.rs", hidden = true)]
//! let a = "";
//!```
//!
//! ```
#![doc = drydoc!(path = "docs/person.rs", toml = { name = "richard" })]
//! ```
//!
// ```
// TODO
// #![doc = drydoc!(path = "docs/import_example.rs")]
// ```

pub use drydoc::drydoc;
