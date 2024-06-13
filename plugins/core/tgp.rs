pub use std::collections::HashMap as StdHashMap;
extern crate lazy_static;
use std::{rc::Rc, sync::Arc};
use std::any::Any;
use std::clone::Clone;
extern crate paste;

use crate::core::comp::{as_static, COMPS, Comp, Param};

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
            None => panic!("missing value for prop {} in profile {:?}",prop,self)
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

pub type FuncType<T> = Arc<dyn Fn(&Arc<Ctx>) -> <T as TgpType>::ResType + Sync + Send>;
pub type FuncTypeNoCtx<T> = Arc<dyn Fn() -> <T as TgpType>::ResType + Sync + Send>;

// pub struct DynmaicProfile<T: TgpType> {
//     ctx: Arc<Ctx>,
//     prop: StaticString,
//     phantom: std::marker::PhantomData<T>,
// }

pub trait TgpType: Any + Send + Sync {
    type ResType;
    fn from_ctx(ctx: &Arc<Ctx>) -> Self::ResType;
}

#[derive(Clone, Debug)]
pub struct Ctx {
    pub path: StaticString,
    pub profile: &'static TgpValue,
    pub comp: Option<&'static Comp>,
    pub parent_param: Option<&'static Param>,
//    pub vars: Vars,
    pub caller_ctx: Option<Arc<Ctx>>,
}

impl Ctx {
    pub fn new(profile: &'static TgpValue) -> Arc<Ctx> { Arc::new( Ctx {profile, parent_param: None, path: "unknown", comp: None, caller_ctx: None } )}
    pub fn new_comp(self: &Arc<Ctx>, comp: &'static Comp) -> Arc<Ctx> { Arc::new(
        Ctx { comp: Some(comp),
        profile: comp.r#impl, parent_param: self.parent_param, 
        path: as_static(&format!("{}~impl",comp.id)),
        caller_ctx: Some(Arc::clone(self))
    }) }
    pub fn run<T: TgpType>(self: &Arc<Ctx>) -> T::ResType {
        let caller_profile : &'static TgpValue = match self.caller_ctx.clone() {
            Some(caller_ctx) => caller_ctx.profile,
            _ => &TgpValue::Nop()
        };
        println!("run ctx {} profile {:?} caller_profile {:?} ", self.path, self.profile, caller_profile);
        match self.profile {
            TgpValue::Profile(prof) => {
                let pt = prof.pt;
                match COMPS.get(pt) {
                    Some(comp) => {
                        match comp.r#impl {
                            TgpValue::RustImpl(ref any_arc) => {
                                match any_arc.downcast_ref::<FuncType<T>>() {
                                    Some(f) => f(self),
                                    None => panic!("can not cast impl func {:?}", self),
                                }
                            },
                            _ => self.new_comp(comp).run::<T>()
                        }
                    },
                    None => panic!("can not find pt {}", pt)
                }                
            },
            TgpValue::Iden(iden) => {
                let profile : &'static TgpValue = match self.caller_ctx.clone() {
                    Some(caller_ctx) => caller_ctx.profile,
                    _ => panic!("no caller ctx {:?}", self)
                };
                match profile {
                    TgpValue::Profile(prof) =>  {
                        let param_id = as_static(&iden.to_string());
                        match prof.props.get(param_id) {
                            Some(tgp_val) => T::from_ctx(&self.profile_and_path(tgp_val, prof.param_def(param_id), self.path)),
                            None => panic!("missing value for prop {} in profile {:?}",param_id,prof)
                        }
                    },
                    _ => panic!("caller_ctx.profile is not a profile {:?}", self),
                }
            },           
            _ => panic!("ctx.run expecting profile as tgpValue {:?}", self)
        }
    }
    pub fn prop<T: TgpType>(self: &Arc<Ctx>, prop: StaticString) -> T::ResType {
        match self.profile {
            TgpValue::Profile(prof) => T::from_ctx(&self.inner_profile(prof, prof.param_def(prop))),
            _ => panic!("ctx.prop '{}' expecting profile as tgpValue {:?}", prop, self)
        }
    }
    pub fn func<T: TgpType>(self: Arc<Ctx>, prop: StaticString) -> FuncTypeNoCtx<T> {
        Arc::new(move || { self.prop::<T>(prop) })
    }
    pub fn profile_and_path(self: &Arc<Ctx>, profile: &'static TgpValue, parent_param: &'static Param, path: &str) -> Arc<Ctx> { Arc::new(
        Ctx { profile, path: as_static(path), parent_param: Some(parent_param), comp: self.comp, 
            caller_ctx: match self.caller_ctx { Some(ref c) => Some(c.clone()), None => None }
        }
    )}
    pub fn inner_profile(self: &Arc<Ctx>, profile: &'static Profile, parent_param: &'static Param) -> Arc<Ctx> {
        let param_id = parent_param.id;
        match profile.props.get(param_id) {
            Some(val) => self.profile_and_path(val, parent_param, &format!("{}~{param_id}", self.path)),
            None => match parent_param.default_value {
                Some(v) => self.profile_and_path(v, parent_param, &format!("{}~params~{param_id}~defaultValue", profile.pt)),
                None => self.profile_and_path(&TgpValue::Nop(), parent_param, &format!("{}~{param_id}", self.path))
            }
        }        
    }
    pub fn inner_profile_in_array(self: &Arc<Ctx>, inner_profile: &'static TgpValue, parent_param: &'static Param, index: usize) -> Option<Arc<Ctx>> {
        let param_id = parent_param.id;
        Some(self.profile_and_path(inner_profile, parent_param, &format!("{}~{param_id}~{index}", self.path)))
    }
}
