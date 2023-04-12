use std::collections::HashMap;

use either::Either;
use numtoa::NumToA;

use crate::prelude::{debug, fmt_err, ErrorCode, InlineStr, TegResult};

#[derive(Clone, Debug)]
pub enum Object {
    Int(i32),
    Long(i64),
    Boolean(bool),
    String(InlineStr),
    Map(HashMap<InlineStr, Object>),
    List(Vec<Object>),
    Null,
}

impl Object {
    pub fn is_null(&self) -> bool {
        match self {
            Self::Null => true,
            _ => false,
        }
    }

    pub fn read(
        document_context: &mut Either<HashMap<InlineStr, Object>, serde_json::Value>,
        path: &str,
    ) -> InlineStr {
        let value = if let Some(value) = document_context.as_ref().right() {
            value
        } else {
            // set value into document_context
            let _ = std::mem::replace(
                document_context,
                Either::Right(Self::convert_hashmap_to_json(
                    document_context.as_ref().left().expect("not none"),
                )),
            );

            document_context.as_ref().right().expect("not none")
        };

        debug!("json for select is: {}", value);
        if let Ok(json) = jsonpath_lib::select(&value, format!("$.{}", path).as_str()) {
            if let Some(&v) = json.get(0) {
                if let serde_json::Value::String(v) = v {
                    return v.into();
                } else {
                    return v.to_string().into();
                }
            }
        }
        "".into()
    }

    pub fn as_bool(&self) -> TegResult<bool> {
        match self {
            Self::Boolean(v) => Ok(*v),
            _ => fmt_err!(UnknownException, "not a bool {:?}", self),
        }
    }

    pub fn as_string(&self) -> TegResult<&InlineStr> {
        match self {
            Self::String(v) => Ok(&v),
            _ => fmt_err!(UnknownException, "not a string {:?}", self),
        }
    }

    pub fn to_string(&self) -> InlineStr {
        match self {
            Object::Int(v) => (*v).numtoa_str(10, &mut [0; 16]).into(),
            Object::Long(v) => (*v).numtoa_str(10, &mut [0; 32]).into(),
            Object::Boolean(v) => {
                if *v {
                    "True".into()
                } else {
                    "False".into()
                }
            }
            Object::String(v) => v.clone(),
            Object::Map(v) => Self::convert_hashmap_to_json(v).to_string().into(),
            Object::List(v) => Self::convert_list_to_json(v).to_string().into(),
            Object::Null => "".into(),
        }
    }

    pub fn estimate_map_memory_used(hashmap: &HashMap<InlineStr, Object>) -> i32 {
        let mut memory_used = 0;
        for (k, v) in hashmap {
            memory_used += k.as_bytes().len() as i32;
            memory_used += v.estimate_memory_used();
        }
        memory_used
    }

    pub fn estimate_list_memory_used(list: &Vec<Object>) -> i32 {
        let mut memory_used = 0;
        for v in list {
            memory_used += v.estimate_memory_used();
        }
        memory_used
    }

    pub fn estimate_memory_used(&self) -> i32 {
        match self {
            Object::Int(_) => 4,
            Object::Long(_) => 8,
            Object::Boolean(_) => 1,
            Object::String(v) => v.as_bytes().len() as i32,
            Object::Map(v) => Self::estimate_map_memory_used(v),
            Object::List(v) => Self::estimate_list_memory_used(v),
            Object::Null => 1,
        }
    }
}

/// json <-> object
impl Object {
    fn convert_hashmap_to_json(hash_map: &HashMap<InlineStr, Object>) -> serde_json::Value {
        let mut map = serde_json::Map::with_capacity(hash_map.len());
        for (k, v) in hash_map {
            map.insert(k.to_string(), v.to_json());
        }
        serde_json::Value::Object(map)
    }

    fn convert_list_to_json(list: &Vec<Object>) -> serde_json::Value {
        let mut json_list = Vec::with_capacity(list.len());
        for v in list {
            json_list.push(v.to_json());
        }
        serde_json::Value::Array(json_list)
    }

    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Object::Int(v) => serde_json::Value::Number((*v).into()),
            Object::Long(v) => serde_json::Value::Number((*v).into()),
            Object::Boolean(v) => serde_json::Value::Bool(*v),
            Object::String(v) => serde_json::Value::String(v.to_string()),
            Object::Map(v) => Self::convert_hashmap_to_json(v),
            Object::List(v) => Self::convert_list_to_json(v),
            Object::Null => serde_json::Value::Null,
        }
    }

    pub fn convert_jsonmap_to_hashmap(
        jsonmap: &serde_json::Map<String, serde_json::Value>,
    ) -> HashMap<InlineStr, Object> {
        let mut map = HashMap::with_capacity(jsonmap.len());
        for (k, v) in jsonmap {
            map.insert(k.into(), Self::from_json(v));
        }
        map
    }

    fn from_json(json: &serde_json::Value) -> Object {
        match json {
            serde_json::Value::Bool(v) => (*v).into(),
            serde_json::Value::Number(v) => {
                if let Some(v) = v.as_i64() {
                    if v < i32::MAX as i64 && v > i32::MIN as i64 {
                        Object::Int(v as i32)
                    } else {
                        Object::Long(v)
                    }
                } else {
                    unimplemented!()
                }
            }
            serde_json::Value::String(v) => v.into(),
            serde_json::Value::Object(v) => Self::convert_jsonmap_to_object(v),
            serde_json::Value::Array(v) => Self::convert_jsonlist_to_object(v),
            serde_json::Value::Null => Object::Null,
        }
    }

    fn convert_jsonmap_to_object(json_map: &serde_json::Map<String, serde_json::Value>) -> Object {
        let mut map = HashMap::with_capacity(json_map.len());
        for (k, v) in json_map {
            map.insert(k.into(), Self::from_json(v));
        }
        Object::Map(map)
    }

    fn convert_jsonlist_to_object(json_list: &Vec<serde_json::Value>) -> Object {
        let mut list = Vec::with_capacity(json_list.len());
        for v in json_list {
            list.push(Self::from_json(v));
        }
        Object::List(list)
    }
}

impl From<i32> for Object {
    fn from(value: i32) -> Self {
        Object::Int(value)
    }
}
impl From<i64> for Object {
    fn from(value: i64) -> Self {
        Object::Long(value)
    }
}
impl From<bool> for Object {
    fn from(value: bool) -> Self {
        Object::Boolean(value)
    }
}
impl From<InlineStr> for Object {
    fn from(value: InlineStr) -> Self {
        Object::String(value)
    }
}
impl From<&InlineStr> for Object {
    fn from(value: &InlineStr) -> Self {
        Object::String(value.clone())
    }
}
impl From<&str> for Object {
    fn from(value: &str) -> Self {
        Object::String(InlineStr::from(value))
    }
}
impl From<&String> for Object {
    fn from(value: &String) -> Self {
        Object::String(InlineStr::from(value))
    }
}
impl From<String> for Object {
    fn from(value: String) -> Self {
        Object::String(InlineStr::from(value))
    }
}
impl From<Vec<Object>> for Object {
    fn from(value: Vec<Object>) -> Self {
        Object::List(value)
    }
}
impl From<HashMap<InlineStr, Object>> for Object {
    fn from(value: HashMap<InlineStr, Object>) -> Self {
        Object::Map(value)
    }
}
