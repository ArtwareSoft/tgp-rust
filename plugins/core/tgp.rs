pub use std::{collections::HashMap as StdHashMap };
use lazy_static::lazy_static;
use serde_json::{Value};
extern crate lazy_static;
use std::collections::{HashSet};
use std::sync::{Arc, Mutex};
use std::any::{Any};
use std::clone::Clone;
use super::rt::{RTValue, Ctx};
use ctor::ctor;
extern crate paste;
use tgp_macro::{tgp_value,tgp_val_from_string, comp};

lazy_static! {
    pub static ref COMPS: Comps = Mutex::new(StdHashMap::new());
    pub static ref DATA_TYPE: TgpValue = TgpValue::String("data");
    pub static ref DATA_PARAM: Param = Param {id: "", r#type: "data", default_value: None, src: &TgpValue::Nop() };
    pub static ref NOP: TgpValue = TgpValue::Nop();
    static ref GLOBAL_TGP_VALUES: Mutex<StdHashMap<StaticString, &'static TgpValue>> = Mutex::new(StdHashMap::new());
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

pub type StaticString = &'static str;

pub fn as_static(input: &str) -> StaticString {
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

#[derive(Debug, Clone)]
pub struct Comp {
    pub id: StaticString,
    pub r#type: StaticString,
    pub params: Vec<Param>,
    pub r#impl: TgpValue,
    pub src: &'static TgpValue,
}
impl Comp {
    pub fn new(src: &'static TgpValue) -> Self {
        Comp { id: src.id(), 
            params: match src.prop("params") {
                Some(v) => match v { TgpValue::Array(ar) => ar.into_iter().map(|x| Param::new(x)).collect() , _ => vec!{}},
                None => vec!{}
            }, 
            r#type: match src.prop("type") {
                Some(v) => match v { TgpValue::String(s) => s , _ => "data"},
                None => "data"
            }, 
            r#impl: src.prop("impl").unwrap().clone(), 
            src 
        }
    }
}
#[derive(Debug, Clone)]
pub struct Param {
    pub id: StaticString,
    pub r#type: StaticString,
    pub default_value: Option<&'static TgpValue>,
    pub src: &'static TgpValue
}

impl Param {
    pub fn new(src: &'static TgpValue) -> Self { 
        Param {
            id: src.id(), 
            r#type: match src.prop("type") {
                Some(v) => match v { TgpValue::String(s) => s , _ => "data"},
                None => "data"
            }, 
            default_value: src.prop("default_value"), 
            src             
        } 
    }
}
impl TgpValue {
    pub fn prop(&self, prop: StaticString) -> Option<&TgpValue> {
        match self {
            TgpValue::Profile(p) => p.props.get(prop),
            _ => None
        }
    }
    pub fn id(&self) -> StaticString {
        let x = match self.prop("id").unwrap() { TgpValue::String(s) => s , _ => ""};
        x
    }
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
    Obj(StdHashMap<StaticString, TgpValue>),
    Nop(),
    Err(String)
}
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
    fn from_json(value: Value) -> TgpValue {
        match value {
            Value::String(s) => TgpValue::String(as_static(&s)),
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

impl Default for TgpValue {
    fn default() -> Self { TgpValue::Nop() }
}

#[ctor]
fn init() {
//     println!("{:?}", tgp_value!(jbComp {
//         id: "pipe",  elems: [3],
//     }
// ));
    //println!("{:?}", tgp_value!(aa{a: "aaa"}));
    println!("{:?}", tgp_val_from_string!("aa{a: 'aaa'}"));
    //println!("{:?}", tgp_value! ( dataTest { calc: 5, expectedResult: equals {to : 5 }} ));
}

pub trait RustImpl: Any + Sync + Send + std::fmt::Debug + 'static {
    fn run(&self, ctx: &Ctx) -> RTValue;
}

// component!( pipeTest, { 
//     type: "data",
//     impl: pipe {source: "a,b,c", operator: [split(","), toUpperCase()]}
// });

// #[ctor]
// fn init() {
//     print!("{}", to_tgp_value! ( dataTest { calc: 5, expectedResult: equals {to : 5 }} ));
// }