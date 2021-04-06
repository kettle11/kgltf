use kjson::*;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct Property {
    title: String,
    description: Option<String>,
    item: Item,
}

#[derive(Debug)]
struct StructDefinition {
    title: String,
    description: Option<String>,
    properties: Vec<Property>,
}

#[derive(Debug)]
enum EnumValue {
    Integer(i64),
    String(String),
    Number(f32),
}

#[derive(Debug)]
struct EnumOption {
    name: String,
    description: Option<String>,
    value: EnumValue,
}

#[derive(Debug)]
struct EnumDefinition {
    title: String,
    description: Option<String>,
    options: Vec<EnumOption>,
}

#[derive(Debug)]

struct Array {
    item_type: Box<Item>,
    min_items: i64,
    max_items: i64,
}

#[derive(Debug)]
enum Item {
    Struct(usize),
    Enum(usize),
    Array(Array),
    Boolean,
    String,
    Integer,
    Number,
    Extension,
    Unknown,
    // Enum(Vec<usize>)
}

struct Parser {
    definitions: HashMap<String, usize>,
    structs: Vec<StructDefinition>,
    enums: Vec<EnumDefinition>,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
            structs: Vec::new(),
            enums: Vec::new(),
        }
    }

    pub fn new_struct(&mut self, title: String, struct_definition: StructDefinition) -> usize {
        println!("NEW STRUCT: {}", title);
        self.definitions.insert(title, self.structs.len());
        self.structs.push(struct_definition);
        self.structs.len() - 1
    }

    pub fn parse_properties(
        &mut self,
        object: &HashMap<String, Value>,
        properties_in: &mut Vec<Property>,
    ) -> Option<()> {
        if let Some(properties) = object.get("properties") {
            let properties = properties.as_object().unwrap();

            let mut properties: Vec<_> = properties.iter().collect();
            properties.sort_by(|a, b| a.0.partial_cmp(b.0).unwrap());

            for (key, property) in properties {
                if key == "extensions" {
                    properties_in.push(Property {
                        title: key.to_string(),
                        description: None,
                        item: Item::Extension,
                    });
                    continue;
                }

                /*
                let property = property.as_object().unwrap();

                let mut item = Item::Unknown;
                if let Some(_type) = property.get("type") {
                    let _type = _type.as_string().unwrap();
                    match _type {
                        "array" => {
                            let items = property.get("items").unwrap();
                            if let Some(item_type) = self.parse_item(items) {
                                item = Item::Array(Box::new(item_type));
                            }
                        }
                        "integer" => {

                        }
                        _ => {
                            println!("UNHANDLED PROPERTY TYPE: {:?}", _type)
                        }
                    }
                } else {
                    println!("DOES NOT HAVE TYPE: {:?}", key);
                }
                */

                /*
                let description = property
                    .get("description")
                    .map(|d| d.as_string().unwrap().to_string());
                */
                /*
                if let Some(all_of) = property.get("allOf") {
                    let json = &all_of.as_array()?[0];
                    item = self.parse_item(json).unwrap();
                }
                */

                println!("PARSING ITEM: {:?}", key);
                let item = self.parse_item(property);

                if let Some(item) = item {
                    properties_in.push(Property {
                        title: key.to_string(),
                        description: None,
                        item,
                    });
                }
            }
        }
        Some(())
    }

    pub fn parse_item(&mut self, value: &Value) -> Option<Item> {
        let object = value.as_object().unwrap();

        // This just refers to a different struct
        if let Some(_ref) = object.get("$ref") {
            let source =
                std::fs::read_to_string(Path::new("schema").join(_ref.as_string().unwrap()))
                    .expect("Could not find file");
            let json = kjson::parse_to_json(&source).expect("Could not parse JSON");
            return self.parse_item(&json);
        }

        if let Some(_type) = object.get("type") {
            match _type.as_string().unwrap() {
                "array" => {
                    let items = object.get("items").unwrap();
                    if let Some(item_type) = self.parse_item(items) {
                        let min_items = object
                            .get("minItems")
                            .map_or(0, |i| i.as_number().unwrap() as i64);
                        let max_items = object
                            .get("maxItems")
                            .map_or(std::i64::MAX, |i| i.as_number().unwrap() as i64);

                        println!("ARRAY ITEMS: {:?}", item_type);
                        let array = Array {
                            min_items,
                            max_items,
                            item_type: Box::new(item_type),
                        };
                        Some(Item::Array(array))
                    } else {
                        Some(Item::Unknown)
                    }
                }
                "object" => {
                    // This is a new struct to define.
                    if let Some(title) = object.get("title") {
                        let title = title.as_string().unwrap();
                        println!("PARSING----------------------: {}", title);

                        if let Some(index) = self.definitions.get(title) {
                            return Some(Item::Struct(*index));
                        }

                        let description = if let Some(description) = object.get("description") {
                            Some(description.as_string().unwrap().to_string())
                        } else {
                            None
                        };

                        let mut properties = Vec::new();
                        self.parse_properties(object, &mut properties);

                        // Extend this object with extensions
                        if let Some(all_of) = object.get("allOf") {
                            let json = &all_of.as_array()?[0];
                            let object = json.as_object()?;
                            self.parse_properties(object, &mut properties);
                        }

                        Some(Item::Struct(self.new_struct(
                            title.to_string(),
                            StructDefinition {
                                title: title.to_string(),
                                description,
                                properties,
                            },
                        )))
                    } else {
                        println!("HERE");
                        None
                    }
                }
                "integer" => Some(Item::Integer),
                "number" => Some(Item::Number),
                "string" => Some(Item::String),
                "boolean" => Some(Item::Boolean),
                _ => {
                    println!("UNKNOWN TYPE: {:?}", _type);
                    Some(Item::Unknown)
                }
            }
        } else {
            if let Some(all_of) = object.get("allOf") {
                let json = &all_of.as_array()?[0];
                return Some(self.parse_item(json).unwrap());
            }

            if let Some(any_of) = object.get("anyOf") {
                let json = &any_of.as_array()?;

                // Oh no this looks messy.
                let _type = json
                    .last()
                    .unwrap()
                    .as_object()
                    .unwrap()
                    .get("type")
                    .unwrap()
                    .as_string()
                    .unwrap();

                let _type = match _type {
                    "integer" => Item::Integer,
                    "number" => Item::Number,
                    "string" => Item::String,
                    "boolean" => Item::Boolean,
                    _ => unimplemented!(),
                };

                let mut options = Vec::new();
                for v in json.iter() {
                    let v = v.as_object().unwrap();

                    let e = if let Some(e) = v.get("enum") {
                        e.as_array().unwrap().first().unwrap()
                    } else {
                        continue;
                    };

                    let description = v
                        .get("description")
                        .map(|d| d.as_string().unwrap().to_string());

                    println!("ENUM GET: {:?}", v.get("enum"));
                    let value = match _type {
                        Item::Integer => EnumValue::Integer(e.as_number().unwrap() as i64),
                        Item::Number => EnumValue::Number(e.as_number().unwrap() as f32),
                        Item::String => EnumValue::String(e.as_string().unwrap().to_string()),
                        _ => unimplemented!(),
                    };
                    options.push(EnumOption {
                        name: description.clone().unwrap_or("Placeholder".to_string()),
                        description,
                        value,
                    });
                }

                self.enums.push(EnumDefinition {
                    title: "placeholder".to_string(),
                    description: None,
                    options,
                });
                return Some(Item::Enum(self.enums.len() - 1));
            }

            None
        }
    }

    fn item_name<'a>(&self, item: &Item) -> String {
        match item {
            Item::Struct(i) => self.structs[*i].title.split_whitespace().collect(),
            Item::String => "String".to_string(),
            Item::Integer => "i64".to_string(),
            Item::Number => "f32".to_string(),
            Item::Array(array) => {
                let item_name = self.item_name(&array.item_type);
                println!("MIN: {:?}, MAX: {:?}", array.min_items, array.max_items);
                if array.max_items == array.min_items {
                    format!("[{}; {:?}]", item_name, array.max_items)
                } else {
                    format!("Vec<{}>", item_name)
                }
            }
            Item::Boolean => "bool".to_string(),
            Item::Extension => "kjson::Value".to_string(),
            Item::Unknown => "unknown".to_string(),
            _ => "unknown".to_string(),
        }
    }

    pub fn write_to_string(&self, output: &mut String) {
        write!(output, "use kjson::Value;\n");

        for s in self.structs.iter().rev() {
            let title: String = s.title.chars().filter(|c| !c.is_whitespace()).collect();

            write!(output, "pub struct {} {{\n", title);
            for Property {
                title,
                description,
                item,
            } in s.properties.iter()
            {
                let _type = self.item_name(&item);
                if let Some(description) = description {
                    write!(output, "    /// {}\n", description);
                }
                write!(output, "    {}: {},\n", title, _type);
            }
            write!(output, "}}\n\n");
        }

        for s in self.enums.iter().rev() {
            let title: String = s.title.chars().filter(|c| !c.is_whitespace()).collect();

            write!(output, "pub enum {} {{\n", title);
            for EnumOption {
                name,
                description,
                value,
            } in s.options.iter()
            {
                if let Some(description) = description {
                    write!(output, "    /// {}\n", description);
                }

                let v = match value {
                    EnumValue::Integer(i) => i.to_string(),
                    EnumValue::String(s) => format!("\"{}\"", s.to_string()),
                    EnumValue::Number(f) => f.to_string(),
                };
                write!(output, "    {} = {},\n", title, v);
            }
            write!(output, "}}\n\n");
        }
    }
}

fn main() {
    let source = std::fs::read_to_string("schema/glTF.schema.json").unwrap();
    let json = kjson::parse_to_json(&source).expect("Could not parse JSON");

    let mut output = String::new();
    let mut parser = Parser::new();
    parser.parse_item(&json);
    parser.write_to_string(&mut output);

    //  println!("{}", output);

    let mut file = File::create("test_output.rs").unwrap();
    file.write_all(output.as_bytes()).unwrap();
}
