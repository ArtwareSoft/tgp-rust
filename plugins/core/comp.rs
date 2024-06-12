use lazy_static::lazy_static;
use std::sync::Mutex;
use crate::core::tgp::{TgpValue,StdHashMap,StaticString,Profile,TgpType,FuncType,Ctx};
use std::collections::HashSet;

use std::sync::Arc;
use tgp_macro::comp;
use ctor::ctor;

lazy_static! {
    pub static ref DSLS: Dsls = Dsls { dsls: Mutex::new(StdHashMap::new())};
    pub static ref COMPS: Comps = Comps { comps: Mutex::new(StdHashMap::new()), unresolved: Mutex::new(Vec::new())};
    pub static ref DATA_TYPE: TgpValue = TgpValue::String("data");
    pub static ref DATA_PARAM: Param = Param::simple("","data", None);
    pub static ref NOP: TgpValue = TgpValue::Nop();
    pub static ref GLOBAL_TGP_VALUES: Mutex<StdHashMap<StaticString, &'static TgpValue>> = Mutex::new(StdHashMap::new());
    static ref GLOBAL_STRINGS: Mutex<HashSet<&'static str>> = Mutex::new(HashSet::new());
    static ref PARAM_OF_PARAM: Param = Param::simple("","param<>",None);
}

pub struct Dsls {
    dsls: Mutex<StdHashMap<StaticString, StaticString>>
}
impl Dsls {
    pub fn insert(&self, module: StaticString, dsl: StaticString) {
        self.dsls.lock().unwrap().insert(module , dsl);
    }
    pub fn dsl_of_module(&self, module: StaticString) -> Option<StaticString> {
        self.dsls.lock().unwrap().get(module).map(|x| *x)
    }
}
pub struct Comps {
    unresolved: Mutex<Vec<(StaticString, StaticString, &'static TgpValue)>>,
    comps: Mutex<StdHashMap<StaticString, &'static Comp>>
}
impl Comps {
    pub fn get(&self, id: &str) -> Option<&'static Comp> {
        self.comps.lock().unwrap().get(id).map(|x| *x)
    }
    pub fn get_impl(&self, id: &str) -> Option<&'static TgpValue> {
        self.comps.lock().unwrap().get(id).map(|comp| comp.r#impl)
    }    
    pub fn add(&self, module: StaticString, id: StaticString, comp: TgpValue) {
        self.unresolved.lock().unwrap().push((module, id, Box::leak(Box::<TgpValue>::from(comp))));
    }
    pub fn resolve(&self) {
        let param = self.unresolved.lock().unwrap().clone().into_iter().find(|(_, short_id, _)| *short_id == "param");
        self.resolve_comp("", "param", param.unwrap().2);
        self.unresolved.lock().unwrap().clone().into_iter().for_each(|(module, short_id, comp)| self.resolve_comp(module, short_id, comp))
    }
    fn resolve_comp(&self, module: StaticString, short_id: StaticString, comp: &'static TgpValue) {
        println!("resolving {}",short_id);
        let dsl = DSLS.dsl_of_module(module).unwrap_or_else(||"");
        let r#type = comp.prop_as_str("type").map_or("data<>", |t| as_static(&format!("{}<{}>", t, dsl)));
        let id = as_static(&format!("{}{}", r#type, short_id));
        let params = comp.prop("params").map_or(vec!{}, |params| match params {
            TgpValue::Array(params) => params.iter().map(|param| {
                let p : &Param = Box::leak(Box::<Param>::from(ParamType::from_ctx_with_dsl(param,dsl)));
                p
            }).collect(),
            _ => vec!{}
        });
        let impl1 = comp.prop("impl").map_or(TgpValue::Nop(), |imp| self.resolve_profile(imp.clone(), &Param::simple("",r#type, None), comp));
        let resolved = Comp {id, module, r#type, params, r#impl: Box::leak(Box::<TgpValue>::from(impl1)), src: comp };
        println!("resolved {:#?}", resolved);
        let mut comps = self.comps.lock().unwrap();
        comps.insert(id , Box::leak(Box::<Comp>::from(resolved)));
    }
    fn resolve_profile(&self, prof: TgpValue, parent_param: &Param, parent_comp: &TgpValue) -> TgpValue {
        println!("resolving profile {:?} {:?}", prof, parent_param);
        match prof {
            TgpValue::UnresolvedProfile(unresolved_pt,values) => {
                let (args_by_value, args_by_name) = match values.last() {
                    Some(TgpValue::Obj(obj)) => (&values[..values.len() - 1], obj.clone()),
                    _ => (&values[..], StdHashMap::new()),
                };
                let pt = as_static(&format!("{}{}", parent_param.r#type, unresolved_pt));
                let new_prof = match self.get(pt) {
                    Some(comp) => {
                        let mut props: StdHashMap<StaticString, TgpValue> = StdHashMap::new();
                        match comp.params_layout() {
                            ParamsLayout::FirstParamAsArray(p1) => {
                                props.insert(p1, TgpValue::Array(args_by_value.into()));
                            },
                            ParamsLayout::SecondParamAsArray(p1,p2) => {
                                props.insert(p1, args_by_value.get(0).unwrap().clone());
                                props.insert(p2, TgpValue::Array((&args_by_value[1..]).into()));
                            },
                            _ => {
                                props = args_by_value.iter().zip(comp.params.iter().map(|p| p.id)).map(|(v,id)| (id,v.clone()))
                                    .collect();
                            },                            
                        };
                        props.extend(args_by_name.into_iter());
                        let resolved_props = props.into_iter().map(|(k,v)| 
                            (k, self.resolve_profile(v, comp.params.iter().find(|p| p.id == k).unwrap(), parent_comp))).collect();
                        Profile::new(pt, resolved_props)
                    },
                    None => TgpValue::Err(as_static(&format!("can not find comp {}", pt)).to_string())
                };
                new_prof
            },
            _ => prof
        }
    }
}

enum ParamsLayout {
    Normal,
    FirstParamAsArray(StaticString),
    SecondParamAsArray(StaticString, StaticString),
}

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
    pub module: StaticString,
    pub r#type: StaticString,
    pub params: Vec<&'static Param>,
    pub r#impl: &'static TgpValue,
    pub src: &'static TgpValue,
}
impl Comp {
    fn params_layout(&self) -> ParamsLayout {
        match self.params.get(0) {
            None => ParamsLayout::Normal,
            Some(first_param) => if first_param.by_name { ParamsLayout::Normal } else {
                match self.params.get(1) {
                    None => if first_param.r#type.contains("[]") { ParamsLayout::FirstParamAsArray(first_param.id) } else { ParamsLayout::Normal},
                    Some(second_param) => if second_param.by_name { ParamsLayout::Normal } else {
                        if second_param.second_param_as_array { ParamsLayout::SecondParamAsArray(first_param.id, second_param.id) } else { ParamsLayout::Normal }
                    }
                }
            }
        }
    }
}
#[derive(Debug, Clone)]
pub struct Param {
    pub id: StaticString,
    pub r#type: StaticString,
    pub default_value: Option<&'static TgpValue>,
    pub by_name: bool,
    pub second_param_as_array: bool,
    pub src: &'static TgpValue
}

impl Param {
    pub fn simple(id: StaticString, r#type: StaticString, default_value: Option<&'static TgpValue>) -> Self { 
        Param { id, r#type, default_value, by_name: false, second_param_as_array: false, src: &TgpValue::Nop() }
    }
    pub fn new(src: &'static TgpValue) -> Self { 
        Param {
            id: src.id().expect("no id for param"), 
            r#type: match src.prop("type") {
                Some(v) => match v { TgpValue::String(s) => s , _ => "data"},
                None => "data"
            }, 
            default_value: src.prop("default_value"),
            by_name: src.prop("byName").map_or(false, |_x| true),
            second_param_as_array: src.prop("secondParamAsArray").map_or(false, |_x| true),
            src             
        } 
    }
    pub fn new_with_dsl(src: &'static TgpValue, dsl: StaticString) -> Self { 
        Param {
            id: src.id().expect(
                &format!("no id for param {:#?}", src)), 
            r#type: src.prop_as_str("type").map_or("data<>", |t| as_static(&format!("{}<{}>", t, dsl))), 
            default_value: src.prop("defaultValue"),
            by_name: src.prop_as_str("byName").map_or(false, |_| true),
            second_param_as_array: src.prop_as_str("secondParamAsArray").map_or(false, |_| true),
            src             
        } 
    }
}

struct ParamType;
impl TgpType for ParamType {
    type ResType = Param;
    fn from_ctx(_ctx: &Ctx) -> Self::ResType { panic!("should be initialized with dsl") }
    fn default_value() -> Param { panic!("no default value for param") }
}

impl ParamType {
    fn from_ctx_with_dsl(profile: &'static TgpValue, dsl: StaticString) -> Param {
        match profile {
            TgpValue::Iden(id) => Param::simple(id, "data<>", None),
            TgpValue::String(id) => Param::simple(id, "data<>", None),
            TgpValue::UnresolvedProfile(_,_) => {
                let param = Box::leak(Box::<TgpValue>::from(COMPS.resolve_profile(profile.clone(), &PARAM_OF_PARAM, profile)));
                Param::new_with_dsl(param, dsl)
            },
            TgpValue::Profile(profile) => panic!("param should be solved as unresolved: {:?}", profile),
            _ => panic!("invalid param: {:?}", profile)
        }
    }
}

pub struct StaticStringType;
impl TgpType for StaticStringType {
    type ResType = StaticString;
    fn from_ctx(ctx: &Ctx) -> Self::ResType {
        match ctx.profile {
            TgpValue::Iden(id) => id,
            TgpValue::String(id) => id,
            _ => panic!("invalid StaticStringType")
        }
    }
    fn default_value() -> StaticString { "" }
}

comp!(param, {
    type: param,
    params: [
        id, "type", "defaultValue" // can not be UnresolvedProfile to avoid endless recustion
    ],
    impl: fn<ParamType> |ctx: &Ctx| { 
        panic!("param should be solved as unresolved: {:?}", ctx);
        Param::simple("id", "type", None)
    }
});
