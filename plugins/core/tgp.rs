pub use std::collections::HashMap as StdHashMap;
extern crate lazy_static;
use std::sync::Arc;
use std::any::Any;
use std::clone::Clone;
extern crate paste;

use crate::core::comp::{as_static, COMPS};

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
    pub fn func<T: TgpType>(&'static self, prop: StaticString) -> DynmaicProfile<T> {
        Arc::new(|| { match self.props.get(prop) {
            Some(v) => T::from_tgp_value(v),
            None => T::default_value()
        }}) as DynmaicProfile<T>
    }
    pub fn prop<T: TgpType>(&'static self, prop: StaticString) -> T::ResType {
        match self.props.get(prop) {
            Some(v) => T::from_tgp_value(v),
            None => T::default_value()
        }
    }
    pub fn calc<T: TgpType>(&'static self) -> T::ResType {        
        let pt = self.pt;
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
    }
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

// pub trait FuncTypeDef<T: TgpType>: Any + Send + Sync {
//     fn calc(&self, profile: &'static Profile) -> T::ResType;
// }

pub type FuncType<T> = Arc<dyn Fn(&'static Profile) -> <T as TgpType>::ResType + Sync + Send>;
pub type DynmaicProfile<T> = Arc<dyn Fn() -> <T as TgpType>::ResType + Sync + Send>;

pub trait TgpType: Any + Send + Sync {
    type ResType;
    fn default_value() -> Self::ResType;
    fn from_tgp_value(profile: &'static TgpValue) -> Self::ResType;
}