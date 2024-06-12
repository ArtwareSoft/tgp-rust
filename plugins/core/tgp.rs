pub use std::collections::HashMap as StdHashMap;
extern crate lazy_static;
use std::{rc::Rc, sync::Arc};
use std::any::Any;
use std::clone::Clone;
extern crate paste;

use crate::core::comp::{as_static, COMPS, Param};

pub type StaticString = &'static str;

impl TgpValue {
    pub fn prop(&self, prop: StaticString) -> Option<&TgpValue> {
        match self {
            TgpValue::Profile(p) => p.props.get(prop),
            _ => None
        }
    }
    pub fn prop_as_str(&self, prop: StaticString) -> Option<StaticString> {
        match self {
            TgpValue::Profile(p) => match p.props.get(prop) {
                Some(TgpValue::String(p)) => Some(p),
                Some(TgpValue::Iden(p)) => Some(as_static(&p.to_string())),
                _ => None
            },
            _ => None
        }
    }    
    pub fn id(&self) -> Option<StaticString> {
        self.prop_as_str("id")
    }
}

#[derive(Debug, Clone)]
pub struct Profile {
    pub pt: StaticString,
    pub props: StdHashMap<StaticString, TgpValue>,
}
impl Profile {
    pub const fn new(pt: StaticString, props: StdHashMap<StaticString, TgpValue>) -> TgpValue { 
        TgpValue::Profile(Profile {pt, props})
    }
    pub fn param_def(&self, param: StaticString) -> &'static Param {
        let pt = self.pt;
        match COMPS.get(pt) {
            Some(comp) => 
                comp.params.iter().find(|p| p.id == param).expect( &format!("can not find param {} in comp {:?}", param, comp) ),
            None => panic!("can not find pt {}", pt)
        }
    }
    // pub fn func<T: TgpType>(&'static self, prop: StaticString) -> DynmaicProfile<T> {
    //     Arc::new(|| { match self.props.get(prop) {
    //         Some(v) => T::from_ctx(v),
    //         None => T::default_value()
    //     }}) as DynmaicProfile<T>
    // }
    pub fn prop<T: TgpType>(&'static self, prop: StaticString) -> T::ResType {
        match self.props.get(prop) {
            Some(v) => T::from_ctx(&Ctx::new(v)),
            None => T::default_value()
        }
    }
    // pub fn calc<T: TgpType>(&'static self) -> T::ResType {        
    //     let pt = self.pt;
    //     match COMPS.get(pt) {
    //         Some(comp) => match comp.r#impl {
    //             TgpValue::RustImpl(ref any_arc) => {
    //                 match any_arc.downcast_ref::<FuncType<T>>() {
    //                     Some(f) => f(self),
    //                     None => panic!("can not cast impl 1 {}", pt),
    //                 }
    //             }
    //             _ => panic!("can not cast impl 2 {}", pt)
    //         },
    //         None => panic!("can not find pt {}", pt)
    //     }        
    // }
}

#[derive(Debug, Clone)]
pub enum TgpValue {
    String(StaticString),
    Int(usize),
    Float(f64),
    Boolean(bool),
    ProfileExtendsCtx(Profile, &'static ExtendCtx),
    Profile(Profile),
    UnresolvedProfile(StaticString, Vec<TgpValue>),
    RustImpl(Arc<dyn Any + Sync + Send + 'static>),
    Array(Vec<TgpValue>),
    Obj(StdHashMap<StaticString, TgpValue>),
    Nop(),
    Iden(StaticString),
    JsFunc(StaticString),
    Err(String)
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

impl Default for TgpValue {
    fn default() -> Self { TgpValue::Nop() }
}

pub type FuncType<T> = Arc<dyn Fn(&Ctx) -> <T as TgpType>::ResType + Sync + Send>;
//pub type DynmaicProfile<T> = Arc<dyn Fn() -> <T as TgpType>::ResType + Sync + Send>;

pub trait TgpType: Any + Send + Sync {
    type ResType;
    fn default_value() -> Self::ResType;
    fn from_ctx(ctx: &Ctx) -> Self::ResType;
}

#[derive(Clone, Debug)]
pub struct Ctx {
    pub profile: &'static TgpValue,
    pub parent_param: Option<&'static Param>,
    pub path: StaticString,
//    pub vars: Vars,
    pub cmp_ctx: Option<Arc<Ctx>>,
}

impl Ctx {
    pub fn new(profile: &'static TgpValue) -> Self { Ctx {profile, parent_param: None, path: "", cmp_ctx: None } }
    pub fn run<T: TgpType>(&self) -> T::ResType {        
        match self.profile {
            TgpValue::Profile(prof) => {
                let pt = prof.pt;
                match COMPS.get(pt) {
                    Some(comp) => match comp.r#impl {
                        TgpValue::RustImpl(ref any_arc) => {
                            match any_arc.downcast_ref::<FuncType<T>>() {
                                Some(f) => f(self),
                                None => panic!("can not cast impl 1 {}", pt),
                            }
                        }
                        _ => panic!("can not cast impl 2 {}", pt)
                    },
                    None => panic!("can not find pt {}", pt)
                }                
            },
            _ => { panic!("ctx.prop expecting profile as tgpValue {:?}", self)}
        }
    }
    pub fn prop<T: TgpType>(&self, prop: StaticString) -> T::ResType {
        match self.profile {
            TgpValue::Profile(prof) => T::from_ctx(&self.inner_profile(prof, prof.param_def(prop))),
            _ => { panic!("ctx.prop expecting profile as tgpValue {:?}", self)}
        }
    }
    // pub fn set_profile(&self, profile: &'static TgpValue) -> Self {
    //     Ctx { profile, parent_param: self.parent_param, path: self.path, cmp_ctx: self.cmp_ctx.as_ref().map(Rc::clone) }
    // }
    pub fn profile_and_path(&self, profile: &'static TgpValue, parent_param: &'static Param, path: &str) -> Self {
        Ctx { profile, path: as_static(path), parent_param: Some(parent_param), cmp_ctx: self.cmp_ctx.as_ref().map(Arc::clone) }
    }
    pub fn inner_profile(&self, profile: &'static Profile, parent_param: &'static Param) -> Self {
        let param_id = parent_param.id;
        match profile.props.get(param_id) {
            Some(val) => self.profile_and_path(val, parent_param, &format!("{}~{param_id}", self.path)),
            None => match parent_param.default_value {
                Some(v) => self.profile_and_path(v, parent_param, &format!("{}~params~{param_id}~defaultValue", profile.pt)),
                None => self.profile_and_path(&TgpValue::Nop(), parent_param, &format!("{}~{param_id}", self.path))
            }
        }        
    }
    pub fn inner_profile_in_array(&self, inner_profile: &'static TgpValue, parent_param: &'static Param, index: usize) -> Option<Self> {
        let param_id = parent_param.id;
        Some(self.profile_and_path(inner_profile, parent_param, &format!("{}~{param_id}~{index}", self.path)))
    }
    // pub fn new_comp(&self, params: HashMap<StaticString, RTValue>, comp: &'static Comp) -> Self {
    //     let pt = comp.id;
    //     Ctx { 
    //         cmp_ctx: Some(Rc::new(self.clone())),
    //         params: Rc::new(params),
    //         path: as_static(&format!("{pt}~impl")),
    //         profile: &comp.r#impl, parent_param: self.parent_param,
    //         data: Rc::clone(&self.data), vars: Rc::clone(&self.vars), 
    //     }
    // }
    // pub fn get_param(&self, param_id: &str) -> Option<RTValue> {
    //     self.params.get(param_id).map_or(None, |v| Some(v.clone()))
    // }
    // pub fn get_string_param(&self, param_id: &str) -> &str {
    //     self.params.get(param_id).map_or("", |v| match v {
    //         RTValue::StaticString(s) => s,
    //         RTValue::DynString(s) => s,
    //         _ => "",
    //     })
    // }
    // pub fn calc_dynamic_param(&self, param_id: &str, data: Option<Data>, vars : Option<Vars>) -> Option<RTValue> {
    //     self.params.get(param_id).map_or(None, |p_value| {
    //         let p_val : &RTValue = &p_value;
    //         match p_val {
    //             RTValue::Func(run_ctx) => Some(jb_run(match (data, vars) {
    //                 (None, None) => run_ctx.clone(),
    //                 (None, Some(vars)) => run_ctx.set_vars(vars),
    //                 (Some(data), None) => run_ctx.set_data(data),
    //                 (Some(data), Some(vars)) => run_ctx.set_data(data).set_vars(vars),
    //             })),
    //             _ => Some(p_val.clone())
    //         }
    //     })
    // }

    // pub fn run_itself(&self) -> RTValue {
    //     jb_run(self.clone())
    // }
    // pub fn get_comp_param(&self, param_id: &str) -> Option<RTValue> {
    //     self.cmp_ctx.as_ref().map_or(None, |cmp_ctx| cmp_ctx.clone().params.clone().get(param_id).map_or(None, |v| Some(v.clone())) )
    // }
    // pub fn extend(&self, extend_ctx: &'static ExtendCtx) -> Ctx {
    //     let data_ctx = match extend_ctx.data {
    //         Some(profile) => self.set_data(Rc::new(jb_run(self.profile_and_path(profile, &DATA_PARAM, &format!("{}/data", self.path))))),
    //         None => self.clone(),
    //     };

    //     let vars_ctx = match extend_ctx.vars {
    //         Some(some_vars_def) => match some_vars_def {
    //             SomeVarsDef::VarsDef(vars ) => {
    //                 let mut new_hash = (*data_ctx.vars).clone();
    //                 for (i, var) in vars.iter().enumerate() {
    //                     new_hash.insert(var.0 , jb_run(
    //                         data_ctx.profile_and_path(var.1.unwrap_or(&NOP), &DATA_PARAM, &format!("{}/$vars/{i}", data_ctx.path))
    //                     ));                            
    //                 }
    //                 data_ctx.set_vars(Rc::new(new_hash))
    //             },
    //             SomeVarsDef::VarDef(id, val ) => {
    //                 let mut new_hash = (*data_ctx.vars).clone();
    //                 new_hash.insert(id , jb_run(
    //                     data_ctx.profile_and_path(val.unwrap_or(&NOP), &DATA_PARAM, &format!("{}/$vars", data_ctx.path))
    //                 ));
    //                 data_ctx.set_vars(Rc::new(new_hash))
    //             }
    //         },
    //         None => data_ctx,
    //     };
    //     vars_ctx
    // }
}
