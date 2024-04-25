use std::{collections::HashMap as StdHashMap};
use lazy_static::lazy_static;
extern crate lazy_static;
use std::collections::HashSet;
use std::sync::Mutex;
use std::any::Any;

lazy_static! {
    pub static ref DATA_PARAM: Param = Param { id: "data", r#type: None, dynamic: None, default_value: None };
    pub static ref COMPS: StdHashMap<StaticString, Comp> = StdHashMap::new();
    pub static ref NOP: TgpValue = TgpValue::Nop();

    static ref GLOBAL_STRINGS: Mutex<HashSet<&'static str>> = Mutex::new(HashSet::new());
}

pub type StaticString = &'static str;

pub fn asStaticString(input: &str) -> StaticString {
    let mut strings = GLOBAL_STRINGS.lock().unwrap();
    if let Some(result) = strings.get(input) {
        return result;
    }
    let result = Box::leak(Box::<str>::from(input));
    strings.insert(result);
    result
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

#[derive(Debug)]
pub struct Param {
    pub id: StaticString,
    pub r#type: Option<StaticString>,
    pub dynamic: Option<bool>,
    pub default_value: Option<TgpValue>
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

#[derive(Debug)]
pub struct Profile {
    pub unresolved_pt: StaticString,
    pub pt: StaticString,
    pub props: StdHashMap<StaticString, &'static TgpValue>,
}
#[derive(Debug)]
pub struct ExtendCtx {
    pub data: Option<&'static TgpValue>,
    pub vars: Option<&'static SomeVarsDef>,
}
#[derive(Debug)]
pub enum SomeVarsDef {
    VarDef(StaticString, Option<&'static TgpValue>),
    VarsDef(Vec<(StaticString, Option<&'static TgpValue>)>),
}

#[derive(Debug)]
pub enum TgpValue {
    StaticString(StaticString),
    String(String),
    I32(i32),
    Boolean(bool),
    Profile(Profile, Option<&'static ExtendCtx>),
    ConstsOnlyProfile(ConstsOnlyProfile),
    CompiledProfile(CompiledProfile),
    Array(Vec<&'static TgpValue>),
    KnownObj(Box<dyn IKnownObj>),
    Nop()
}

impl Default for TgpValue {
    fn default() -> Self { TgpValue::Nop() }
}
#[derive(Debug)]
pub struct ConstsOnlyProfile {
    pub unresolved_pt: StaticString,
    pub pt: StaticString,
    pub props: StdHashMap<StaticString, TgpValue>,
    pub cached_params: &'static StdHashMap<StaticString, TgpValue>,
}

#[derive(Debug)]
pub struct CompiledProfile {
    pub unresolved_pt: StaticString,
    pub pt: StaticString,
    pub props: StdHashMap<StaticString, TgpValue>,
    pub compiled: &'static Box<dyn IKnownObj>,
}

use super::rt::{RTValue, Ctx};

pub trait IKnownObj: Any + Sync + Send {
    fn query_interface(&self, interface: &str) -> Option<fn(ctx: Ctx, method_name: Option<&str>) -> (Ctx, RTValue)>;
    fn debug_info(&self) -> String;
}

impl std::fmt::Debug for dyn IKnownObj + 'static {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.debug_info())
    }
}

/*
To implement
comp('DirectoryToComps', {
    params: [
        {id: 'basePath', as: 'string' },
        {id: 'topPlugins', as: 'array' },
        {id: 'modelOnly', as: 'boolean' },
    ],
    impl: pipe(
        directoryContent('%$basePath%','*.js|*.tgp'),
        filesToPlugins(),
        extend('using', pipeline('%files/content%', tgpProp('using'), unique())),
        sortByOneToManyRelation('id','using'),
        concatMap(
            Var('plugin'),
            '%files%',
            Var('dsl', tgpProp('dsl')),
            splitComps(),
            extendWithParser('id','type','params', 'dsl', { parser: compHeader() } ),
            resolveTypeAndExtendTgpModel(), // cmps->cmps with $tgpModel
        )
        if('%$modelOnly%','%%', parseComp(cmpParser())) // cmp -> cmp with tgpModel 
    )
})
*/
