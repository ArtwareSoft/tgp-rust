use std::rc::Rc;
use std::{collections::HashMap as StdHashMap };
use lazy_static::lazy_static;
extern crate lazy_static;
use std::collections::{HashSet, HashMap};
use std::sync::{Arc, Mutex};
use std::any::Any;
use std::clone::Clone;
use ctor::ctor;

lazy_static! {
    pub static ref COMPS: Comps = Mutex::new(StdHashMap::new());
    pub static ref DATA_PARAM: Param = Param { id: "data", r#type: None, dynamic: false, default_value: None };
    pub static ref NOP: TgpValue = TgpValue::Nop();

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

pub trait RustImpl: Any + Sync + Send + 'static {
    fn run(&self, ctx: &Ctx) -> RTValue;
    fn debug_info(&self) -> String;
}

#[ctor]
fn init() { 
    COMPS.add("Same", Comp { id: "split", r#type: "data", params: vec![], r#impl: TgpValue::Nop() })
}

lazy_static! {
    static ref MODULE_INIT: () = { 
        COMPS.add("Same", Comp { id: "split", r#type: "data", params: vec![],  r#impl: TgpValue::Nop() });
    };
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
pub struct TgpModel {
    pub comps: StdHashMap<StaticString, Comp>,
    pub plugins: Vec<Plugin>
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
    pub dynamic: bool,
    pub default_value: Option<&'static TgpValue>
}

impl Param {
    pub fn new(id: StaticString) -> Self { Param {id, r#type: Some("data"), dynamic: false, default_value: None} }
}

#[derive(Debug)]
pub struct Plugin {
    comps: Vec<Comp>,
    base_dir: StaticString,
    dsl: StaticString,
    files: Vec<String>,
    dependent: Vec<StaticString>
}

#[derive(Debug)]
struct RawPluginDir {
    base_dir: String,
    files: Vec<File>,
}
#[derive(Debug)]
struct RawFile {
    path: String,
    content: String,
}
#[derive(Debug)]
struct File {
    path: String,
    content: String,
    using: Vec<StaticString>,
    dsl: StaticString,
    plugin_dsl: StaticString
}


#[derive(Debug, Clone)]
pub struct Profile {
    pub pt: StaticString,
    pub props: StdHashMap<StaticString, TgpValue>,
    pub unresolved_pt: StaticString,
}
impl Profile {
    pub const fn new(pt: StaticString, props: StdHashMap<StaticString, TgpValue>) -> TgpValue { 
        TgpValue::Profile(Profile {pt, props, unresolved_pt: "" } , None)
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
    StaticString(StaticString),
    String(String),
    I32(i32),
    Boolean(bool),
    Profile(Profile, Option<&'static ExtendCtx>),
    ConstsOnlyProfile(ConstsOnlyProfile),
    RustImpl(Arc<dyn RustImpl>),
    Array(Vec<TgpValue>),
    Nop()
}

impl Default for TgpValue {
    fn default() -> Self { TgpValue::Nop() }
}
#[derive(Debug, Clone)]
pub struct ConstsOnlyProfile {
    pub unresolved_pt: StaticString,
    pub pt: StaticString,
    pub props: StdHashMap<StaticString, TgpValue>,
}

use super::rt::{RTValue, Ctx};

impl std::fmt::Debug for dyn RustImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.debug_info())
    }
}

