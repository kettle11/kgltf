//! A minimal crate for loading glTF.
//! 
//! This crate is auto-generated from the specification's Json Schema,
//! so some comments may not exactly match the Rust names.

mod gltf_from_json;
mod glb;

pub use glb::*;
pub use gltf_from_json::*;
