pub use std::{collections::HashMap as StdHashMap };
use lazy_static::lazy_static;
use serde_json::{Map, Number, Value};
extern crate lazy_static;
use std::collections::{HashSet};
use std::sync::{Arc, Mutex};
use std::any::Any;
use std::clone::Clone;
use super::rt::{RTValue, Ctx};

lazy_static! {
    pub static ref COMPS: Comps = Mutex::new(StdHashMap::new());
    pub static ref DATA_PARAM: Param = Param { id: "data", r#type: None, default_value: None };
    pub static ref NOP: TgpValue = TgpValue::Nop();
    static ref GLOBAL_TgpValues: Mutex<StdHashMap<StaticString, &'static TgpValue>> = Mutex::new(StdHashMap::new());
    static ref GLOBAL_STRINGS: Mutex<HashSet<&'static str>> = Mutex::new(HashSet::new());
}

pub trait CompsTrait {
    fn get(&self, id: &str) -> Option<&'static Comp>;
    fn add(&self, id: &'static str, comp: Comp);
}
pub type Comps = Mutex<StdHashMap<StaticString, &'static Comp>>;
impl CompsTrait for Comps {
    fn get(&self, id: &str) -> Option<&'static Comp> {
        let comps = self.lock().unwrap();
        match comps.get(id) { Some(x) => Some(x), None => None }
    }
    fn add(&self, id: &'static str, comp: Comp) {
        let mut comps = self.lock().unwrap();
        comps.insert(id , Box::leak(Box::<Comp>::from(comp)));
    }    
}

pub trait RustImpl: Any + Sync + Send + std::fmt::Debug + 'static {
    fn run(&self, ctx: &Ctx) -> RTValue;
}

pub type StaticString = &'static str;

pub fn asStaticString(input: &str) -> StaticString {
    let mut strings = GLOBAL_STRINGS.lock().unwrap();
    match strings.get(input) {
        Some(result) => result,
        None => {
            let result = Box::leak(Box::<str>::from(input));
            strings.insert(result);
            result        
        }
    }
}

#[derive(Debug)]
pub struct Comp {
    pub r#type: StaticString,
    pub id: StaticString,
    pub params: Vec<Param>,
    pub r#impl: TgpValue,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub id: StaticString,
    pub r#type: Option<StaticString>,
    pub default_value: Option<&'static TgpValue>
}

impl Param {
    pub fn new(id: StaticString) -> Self { Param {id, r#type: Some("data"), default_value: None} }
}

#[derive(Debug, Clone)]
pub struct Profile {
    pub pt: StaticString,
    pub props: StdHashMap<StaticString, TgpValue>,
}
impl Profile {
    pub const fn new(pt: StaticString, props: StdHashMap<StaticString, TgpValue>) -> TgpValue { 
        TgpValue::Profile(Profile {pt, props })
    }
}

#[derive(Debug, Clone)]
pub struct ExtendCtx {
    pub data: Option<&'static TgpValue>,
    pub vars: Option<&'static SomeVarsDef>,
}
#[derive(Debug, Clone)]
pub enum SomeVarsDef {
    VarDef(StaticString, Option<&'static TgpValue>),
    VarsDef(Vec<(StaticString, Option<&'static TgpValue>)>),
}

#[derive(Debug, Clone)]
pub enum TgpValue {
    String(StaticString),
    I32(i32),
    F64(f64),
    Boolean(bool),
    ProfileExtendsCtx(Profile, &'static ExtendCtx),
    Profile(Profile),
    RustImpl(Arc<dyn RustImpl>),
    Array(Vec<TgpValue>),
    Nop(),
    Err(String)
}
impl TgpValue {
    pub fn parse_json(json: &str) -> &'static TgpValue {
        let globals_ro = GLOBAL_TgpValues.lock().unwrap();
        match globals_ro.get(json) { 
            Some(x) => x, 
            None => {
                drop(globals_ro);
                let val = TgpValue::from_json(serde_json::from_str(json).unwrap());
                let st_val = Box::leak(Box::<TgpValue>::from(val));
                let mut globals = GLOBAL_TgpValues.lock().unwrap();
                globals.insert(asStaticString(json) , st_val);
                st_val
            }
        }
    }
    fn from_json(value: Value) -> TgpValue {
        match value {
            Value::String(s) => TgpValue::String(asStaticString(&s)),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    TgpValue::I32(i as i32)
                } else if let Some(i) = n.as_f64() {
                    TgpValue::F64(i as f64)
                } else {
                    TgpValue::Err("Unsupported number type".to_string())
                }
            },
            Value::Bool(b) => TgpValue::Boolean(b),
            Value::Object(obj) => {
                let props: StdHashMap<StaticString, TgpValue> = obj.clone().into_iter().filter(|(key, _)| "$$" != key)
                    .map(|(key, value)| (asStaticString(&key), TgpValue::from_json(value))).collect();
                let pt = match obj.get("$$").unwrap() { Value::String(s) => asStaticString(s), _ => "" };
                TgpValue::Profile(Profile { pt, props })        
            },
            Value::Array(ar) => TgpValue::Array(ar.into_iter().map(|v| TgpValue::from_json(v)).collect()),
            _ => TgpValue::Err(format!("Unsupported json type: {}", value))
        }
    }
}

impl Comp {
    pub fn parse_json(json: &str) -> &'static TgpValue {
        let globals_ro = GLOBAL_TgpValues.lock().unwrap();
        match globals_ro.get(json) { 
            Some(x) => x, 
            None => {
                drop(globals_ro);
                let val = TgpValue::from_json(serde_json::from_str(json).unwrap());
                let st_val = Box::leak(Box::<TgpValue>::from(val));
                let mut globals = GLOBAL_TgpValues.lock().unwrap();
                globals.insert(asStaticString(json) , st_val);
                st_val
            }
        }
    }
}

impl Default for TgpValue {
    fn default() -> Self { TgpValue::Nop() }
}
#[derive(Debug, Clone)]
pub struct ConstsOnlyProfile {
    pub pt: StaticString,
    pub props: StdHashMap<StaticString, TgpValue>
}
