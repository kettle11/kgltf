use kgltf::*;
use kjson::*;

fn main() {
    let file = std::fs::read_to_string("cube/Cube.gltf").unwrap();
    let gltf = GlTf::from_json(&file);
    println!("{:#?}", gltf);
}
