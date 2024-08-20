use serde_json::Value;
use super::{comp1::{as_static, Comp, GLOBAL_TGP_VALUES}, tgp1::{Profile, StaticString, StdHashMap, TgpValue} };

impl TgpValue {
    pub fn parse_json(json: &str) -> &'static TgpValue {
        let globals_ro = GLOBAL_TGP_VALUES.lock().unwrap();
        match globals_ro.get(json) { 
            Some(x) => x, 
            None => {
                drop(globals_ro);
                let val = TgpValue::from_json(serde_json::from_str(json).unwrap());
                let st_val = Box::leak(Box::<TgpValue>::from(val));
                let mut globals = GLOBAL_TGP_VALUES.lock().unwrap();
                globals.insert(as_static(json) , st_val);
                st_val
            }
        }
    }
    pub fn from_json(value: Value) -> TgpValue {
        match value {
            Value::String(s) => TgpValue::String(as_static(&s)),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    TgpValue::Int(i as usize)
                } else if let Some(i) = n.as_f64() {
                    TgpValue::Float(i as f64)
                } else {
                    TgpValue::Err("Unsupported number type".to_string())
                }
            },
            Value::Bool(b) => TgpValue::Boolean(b),
            Value::Object(obj) => {
                let props: StdHashMap<StaticString, TgpValue> = obj.clone().into_iter().filter(|(key, _)| "$$" != key)
                    .map(|(key, value)| (as_static(&key), TgpValue::from_json(value))).collect();
                let pt = obj.get("$$").map(|v| match v { Value::String(s) => as_static(s), _ => "" });
                match pt {
                    Some(pt) => TgpValue::Profile(Profile { pt, props }),
                    None => TgpValue::Obj(props)
                }
            },
            Value::Array(ar) => TgpValue::Array(ar.into_iter().map(|v| TgpValue::from_json(v)).collect()),
            _ => TgpValue::Err(format!("Unsupported json type: {}", value))
        }
    }
}

impl Comp {
    pub fn parse_json(json: &str) -> &'static TgpValue {
        let globals_ro = GLOBAL_TGP_VALUES.lock().unwrap();
        match globals_ro.get(json) { 
            Some(x) => x, 
            None => {
                drop(globals_ro);
                let val = TgpValue::from_json(serde_json::from_str(json).unwrap());
                let st_val = Box::leak(Box::<TgpValue>::from(val));
                let mut globals = GLOBAL_TGP_VALUES.lock().unwrap();
                globals.insert(as_static(json) , st_val);
                st_val
            }
        }
    }
}