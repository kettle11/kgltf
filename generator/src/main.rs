use kjson::*;
use std::collections::HashMap;
use std::path::Path;
use std::{borrow::Cow, usize};

struct StructProperty {
    name: String,
    value_type: Value,
}
struct Struct {
    name: Option<String>,
    description: Option<String>,
    properties: Vec<StructProperty>,
}

type StructHandle = usize;
enum ValueType {
    Struct(StructHandle),
    Array { item_type: Box<ValueType> },
    Object,
    I32,
    String,
    F32,
    Bool,
}

struct Value {
    value_type: ValueType,
    description: Option<String>,
}

struct Generator {
    structs: Vec<Struct>,
    file_to_struct: HashMap<String, StructHandle>,
}

impl Generator {
    fn new() -> Self {
        Self {
            structs: Vec::new(),
            file_to_struct: HashMap::new(),
        }
    }
    fn new_struct(&mut self, name: String, s: Struct) -> StructHandle {
        let index = self.structs.len();
        self.file_to_struct.insert(name, index);
        self.structs.push(s);
        index
    }

    fn generate_struct_from_file(&mut self, file: &str) -> StructHandle {
        println!("GENERATING FROM FILE: {}", file);
        if let Some(path) = self.file_to_struct.get(file) {
            return *path;
        }
        let source =
            std::fs::read_to_string(Path::new("schema").join(file)).expect("Could not find file");
        let json = Thing::from_json(&source).expect("Could not parse JSON");
        let new_struct = self.generate_struct(json.object().unwrap());
        match new_struct.value_type {
            ValueType::Struct(i) => i,
            _ => panic!("Expected struct"),
        }
    }

    fn extend<'a>(&mut self, value: &mut Value, thing: &Thing<'a>) {}

    fn generate_struct<'a>(&mut self, source: &HashMap<Cow<'a, str>, Thing<'a>>) -> Value {
        if let Some(r) = source.get("$ref") {
            let path = r.string().unwrap();
            return Value {
                value_type: ValueType::Struct(self.generate_struct_from_file(&path)),
                description: None,
            };
        }

        let title = source.get("title").map(|s| s.string().unwrap().to_string());
        println!("TITLE: {:?}", title);
        let description = source
            .get("description")
            .map(|s| s.string().unwrap().to_string());

        println!("SOURCE: {:#?}", source);
        println!("TITLE: {:?}", title);
        println!("DESCRIPTION: {:?}", description);

        let mut value = Value {
            value_type: ValueType::Object,
            description: description.clone(),
        };

        if let Some(type_) = source.get("type") {
            let type_ = type_.string().unwrap();
            value.value_type = match &**type_ {
                "object" => {
                    let properties = source.get("properties").unwrap().object().unwrap();
                    let mut properties: Vec<_> = properties.into_iter().collect();
                    properties.sort_by_key(|(name, _)| &**name);

                    for p in properties.iter() {
                        println!("PROPERTY: {}", &p.0);
                    }
                    let mut struct_properties = Vec::new();
                    for (name, thing) in properties {
                        println!("NAME: {}", name);
                        let s = self.generate_struct(thing.object().unwrap());
                        struct_properties.push(StructProperty {
                            name: name.to_string(),
                            value_type: s,
                        });
                    }

                    let title = title.unwrap();
                    let new_struct_id = self.new_struct(
                        title.clone(),
                        Struct {
                            name: Some(title),
                            description: description.clone(),
                            properties: struct_properties,
                        },
                    );

                    ValueType::Struct(new_struct_id)
                }
                "array" => {
                    let items = source.get("items").unwrap().object().unwrap();
                    let item = self.generate_struct(items);
                    ValueType::Array {
                        item_type: Box::new(item.value_type),
                    }
                }
                "integer" => ValueType::I32,
                "number" => ValueType::F32,
                "boolean" => ValueType::Bool,
                "string" => ValueType::String,
                _ => panic!("Unknown type: {:?}", type_),
            }
        }
        /*else {
            // This is a unique case that can be anything, so declare a special object type for it.
            let title = title.unwrap();
            let struct_properties = vec![StructProperty {
                name: "value".to_string(),
                value_type: Value {
                    description: None,
                    value_type: ValueType::Object,
                },
            }];

            let new_struct_id = self.new_struct(
                title.clone(),
                Struct {
                    name: Some(title),
                    description: description.clone(),
                    properties: struct_properties,
                },
            );
            ValueType::Struct(new_struct_id)
        };
        */
        if let Some(all_of) = source.get("allOf") {
            let all_of = all_of.array().unwrap();
            for extend_with in all_of.iter() {
                self.extend(&mut value, extend_with);
            }
        }

        value
    }
}

fn main() {
    let source = std::fs::read_to_string("schema/glTF.schema.json").unwrap();
    let json = kjson::Thing::from_json(&source).expect("Could not parse JSON");

    let mut generator = Generator::new();
    if let Thing::Object(o) = json {
        generator.generate_struct_from_file("glTF.schema.json");
    }
    // println!("{:#?}", json);
    /*
    let mut output = String::new();
    let mut parser = Parser::new();
    parser.parse_item(&json);
    parser.write_to_string(&mut output);

    //  println!("{}", output);

    let mut file = File::create("test_output.rs").unwrap();
    file.write_all(output.as_bytes()).unwrap();
    */
}
