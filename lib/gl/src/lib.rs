#![allow(bare_trait_objects)]
use std::rc::Rc;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub type GlPtr = Rc<Gl>;