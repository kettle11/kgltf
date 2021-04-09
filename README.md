**WORK IN PROGRESS** 

`kgltf` is a minimalist Rust crate for loading glTF files.

[glTF](https://www.khronos.org/gltf/) is an open standard for transmitting and loading 3D scenes and models.

--- 

 `kgltf` is autogenerated from the official glTF [Json specification](https://github.com/KhronosGroup/glTF/tree/master/specification/2.0). 

`kgltf` matches very closely to the specification, even its documentation comments are pulled from the specification. This makes it a useful tool to familiarize yourself with `glTF`.

`kgltf` is designed for ultra-snappy build times. A clean build of `kgltf` takes about 1 second on an M1 Macbook Air.

---

`kgltf` is *not* a seamless solution for loading 3D models / scenes. This crate closely matches the specification. This design makes it very simple and flexible, but you'll have additional code to use it within a codebase.

`kgltf` is also very new, it's barely tested and everything is subject to change.

---

To regenerate the `gltf_json.rs` file navigate to the `generator` directory and run `cargo run`.