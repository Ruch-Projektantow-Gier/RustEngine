#![allow(bare_trait_objects)]
use std::rc::Rc;

mod bindings;
pub use crate::bindings::*;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub type GlPtr = Rc<crate::Gl>;

