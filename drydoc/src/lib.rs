//! Drydoc
//!
//! Split up your documentation compontents into files
//!
//!```
#![doc = crate::doc_hide!("docs/person.rs")]
//!```

pub use drydoc_derive::doc_hide;
pub use drydoc_derive::doc_show;
