use kjson::*;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct SchemaObject {
    properties: HashMap<String, Schema>,
    additional_properties: Option<Box<Schema>>,
    required_properties: HashSet<String>,
    // property_names is unimplemented
    min_properties: Option<usize>,
    max_properties: Option<usize>,
    // Schema dependencies are unimplemented
    // dependencies: HashSet<String, Vec<String>>,
}

#[derive(Debug)]
struct SchemaArray {
    items: Option<Box<Schema>>,
    min_items: Option<usize>,
    max_items: Option<usize>,
    unique_items: bool,
}

#[derive(Debug)]
enum SchemaValue {
    None,
    SchemaObject(SchemaObject),
    SchemaArray(SchemaArray),
    String,
    // Range
    Integer(i64, i64),
    Number,
    Boolean,
}

#[derive(Debug)]
enum SchemaCombination {
    None,
    AnyOf(Vec<Schema>),
    AllOf(Vec<Schema>),
    Not(Vec<Schema>),
    OneOf(Vec<Schema>),
}

#[derive(Debug)]
struct Schema {
    title: Option<String>,
    description: Option<String>,
    schema_value: SchemaValue,
    schema_combination: SchemaCombination,
}

struct Parser {
    root: String,
    files: HashMap<PathBuf, Value>,
}

impl Parser {
    fn new(root: &str) -> Self {
        Self {
            root: root.to_string(),
            files: HashMap::new(),
        }
    }

    fn extend_schema(
        &mut self,
        object: &HashMap<String, Value>,
        extending: &mut Schema,
    ) -> Option<()> {
        if let Some(_ref) = object.get("$ref") {
            println!("REF: {:?}", _ref);

            // Load and parse the reference schema
            let source = std::fs::read_to_string(Path::new(&self.root).join(_ref.as_string()?))
                .expect("Could not find file");
            let json = kjson::parse_to_json(&source).expect("Could not parse JSON");

            self.extend_schema(json.as_object()?, extending);
        }

        if extending.description.is_none() {
            if let Some(description) = object.get("description") {
                extending.description = Some(description.as_string()?.to_string());
            }
        }

        if extending.title.is_none() {
            if let Some(title) = object.get("title") {
                println!("NEW TITLE: {:?}", title);
                extending.title = Some(title.as_string()?.to_string());
            }
        }

        if let Some(_type) = object.get("type") {
            extending.schema_value = match _type.as_string()? {
                "array" => {
                    let items = if let Some(items) = object.get("items") {
                        //   println!("PARSE ITEMS for {:?}", description);
                        Some(Box::new(self.parse_schema(items.as_object()?)?))
                    } else {
                        None
                    };

                    let min_items = if let Some(min_items) = object.get("minItems") {
                        Some(min_items.as_number()? as usize)
                    } else {
                        None
                    };
                    let max_items = if let Some(max_items) = object.get("maxItems") {
                        Some(max_items.as_number()? as usize)
                    } else {
                        None
                    };

                    let unique_items = if let Some(unique_items) = object.get("uniqueItems") {
                        unique_items.as_boolean()?
                    } else {
                        false
                    };

                    SchemaValue::SchemaArray(SchemaArray {
                        items,
                        min_items,
                        max_items,
                        unique_items,
                    })
                }
                "string" => SchemaValue::String,
                "integer" => {
                    let minimum = if let Some(minimum) = object.get("minimum") {
                        let minimum = minimum.as_number()?;
                        minimum as i64
                    } else {
                        i64::MIN
                    };

                    let maximum = if let Some(maximum) = object.get("maximum") {
                        let maximum = maximum.as_number()?;
                        maximum as i64
                    } else {
                        i64::MAX
                    };
                    SchemaValue::Integer(minimum, maximum)
                }
                "number" => SchemaValue::Number,
                "boolean" => SchemaValue::Boolean,
                "object" => SchemaValue::SchemaObject(SchemaObject {
                    properties: HashMap::new(),
                    additional_properties: None,
                    required_properties: HashSet::new(),
                    min_properties: None,
                    max_properties: None,
                    // dependencies: HashSet::<String, Vec<String>>::new(),
                }),
                s => {
                    println!("GENERATING NONE: {:?}", s);
                    SchemaValue::None
                }
            };
        }

        if let Some(all_of) = object.get("allOf") {
            println!("ALL OF");
            let all_of_array = all_of.as_array()?;
            for o in all_of_array.iter() {
                let o = o.as_object()?;
                self.extend_schema(o, extending)?;
            }
        }

        if let Some(any_of) = object.get("any_of") {
            let mut any_of_vec = Vec::new();
            let any_of_array = any_of.as_array()?;
            for o in any_of_array.iter() {
                any_of_vec.push(self.parse_schema(o.as_object()?)?);
            }
            extending.schema_combination = SchemaCombination::AnyOf(any_of_vec);
        }

        if let SchemaValue::SchemaObject(extending) = &mut extending.schema_value {
            if let Some(properties) = object.get("properties") {
                for (key, property) in properties.as_object()?.iter() {
                    if key == "asset" {
                        println!("PARSING ASSET----------- {:#?}", property);
                    }
                    let schema = self.parse_schema(property.as_object()?)?;
                    if key == "asset" {
                        println!("ASSET SCHEMA----------- {:#?}", schema);

                    }
                    extending.properties.insert(key.to_string(), schema);
                }
            }

            if let Some(additional_properties) = object.get("additionalProperties") {
                // additionalProperties can also be a bool, but that's not handled here.
                let schema = self.parse_schema(additional_properties.as_object()?)?;
                extending.additional_properties = Some(Box::new(schema));
            }

            if let Some(Value::Array(required_array)) = object.get("required") {
                for key in required_array.iter() {
                    extending
                        .required_properties
                        .insert(key.as_string()?.to_string());
                }
            }
        }

        Some(())
    }

    fn parse_schema(&mut self, object: &HashMap<String, Value>) -> Option<Schema> {
        let mut schema = Schema {
            description: None,
            title: None,
            schema_value: SchemaValue::None,
            schema_combination: SchemaCombination::None,
        };

        self.extend_schema(object, &mut schema)?;
        
        println!("DONE PARSING SCHEMA");
        Some(schema)
    }
}

fn get_type_name(schema: &Schema) -> String {
    (if let Some(title) = &schema.title {
        println!("TITLE: {:?}", title);
        title
    } else {
        match &schema.schema_value {
            SchemaValue::String => "String",
            SchemaValue::Integer(0, _) => "usize",
            SchemaValue::Integer(0, 255) => "u8",
            SchemaValue::Integer(_, _) => "i64",
            SchemaValue::Number => "f64",
            _ => "unknown", // _ => unimplemented!("{:#?}", &schema),
        }
    })
    .chars()
    .filter(|c| !c.is_whitespace())
    .collect()
}

fn generate_from_schema(schema: &Schema, output: &mut String, generated: &mut HashSet<String>) {
    match &schema.schema_value {
        SchemaValue::SchemaObject(object) => {
            let mut generate_next = Vec::new();
            let mut properties = Vec::new();
            if object.properties.len() > 0 {
                if let Some(title) = &schema.title {
                    let title: String = title.chars().filter(|c| !c.is_whitespace()).collect();
                    // A struct
                    if let Some(description) = &schema.description {
                        write!(output, "/// {} \n", description);
                    }

                    if title == "MaterialNormalTextureInfo" {
                        println!("SCHEMA: {:#?}", schema);
                    }

                    write!(output, "pub struct {} {{\n", title);
                    properties = object.properties.iter().collect();
                    properties.sort_by(|a, b| a.0.partial_cmp(b.0).unwrap());

                    for (name, property) in properties.iter() {
                        //  println!("DESCRIPTION: {:?}", &property);

                        if let Some(description) = &property.description {
                            write!(output, "    /// {} \n", description);
                        }
                        match &property.schema_value {
                            SchemaValue::SchemaObject(property_object) => {
                                let _type = get_type_name(&property);
                                if !generated.contains(&_type) {
                                    generated.insert(_type.clone());
                                    generate_next.push(&**property);
                                }
                                write!(output, "    pub {}: {},\n", name, _type);
                            }
                            SchemaValue::SchemaArray(array) => {
                                let _type = get_type_name(array.items.as_ref().unwrap());

                                match array.items.as_ref().unwrap().schema_value {
                                    SchemaValue::SchemaObject(_) => {
                                        if !generated.contains(&_type) {
                                            generated.insert(_type.clone());
                                            generate_next.push(&array.items.as_ref().unwrap());
                                        }
                                    }
                                    _ => {}
                                }

                                println!("VEC TYPE NAME: {:?}", _type);
                                write!(output, "    pub {}: Vec<{}>,\n", name, _type);
                            }
                            SchemaValue::None => {
                                println!("NONE: {}", name);
                            }
                            SchemaValue::Integer(0, 255) => {
                                write!(output, "    pub {}: u8,\n", name);
                            }
                            SchemaValue::Integer(0, _) => {
                                write!(output, "    pub {}: usize,\n", name);
                            }
                            SchemaValue::Integer(_, _) => {
                                write!(output, "    pub {}: i64,\n", name);
                            }
                            SchemaValue::String => {
                                write!(output, "    pub {}: String,\n", name);
                            }
                            SchemaValue::Boolean => {
                                write!(output, "    pub {}: bool,\n", name);
                            }
                            SchemaValue::Number => {
                                write!(output, "    pub {}: f64,\n", name);
                            }
                            //_ => {}
                            _ => unimplemented!("{:?}", &property.schema_value),
                        }
                    }
                    write!(output, "}}\n");
                }
            } else {
                // Probably a dictionary or an enum
                if 
            }

            for schema in generate_next.iter() {
                write!(output, "\n");
                generate_from_schema(schema, output, generated);
            }
        }
        _ => unimplemented!("{:?}", schema.schema_value),
    }
}

fn main() {
    let source = std::fs::read_to_string("schema/glTF.schema.json").unwrap();
    let json = kjson::parse_to_json(&source).expect("Could not parse JSON");

    let mut parser = Parser::new("schema");

    let schema = parser.parse_schema(&json.as_object().unwrap()).unwrap();

    // println!("HERE: {:#?}", schema);
    println!("MADE IT HERE");

    let mut output = String::new();
    let mut generated = HashSet::new();
    generate_from_schema(&schema, &mut output, &mut generated);
    let mut file = File::create("test_output.rs").unwrap();
    file.write_all(output.as_bytes()).unwrap();
    println!("DONE");
}
