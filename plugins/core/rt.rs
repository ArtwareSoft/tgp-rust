use std::sync::Arc;
use std::{collections::HashMap as StdHashMap, rc::Rc};
use std::collections::HashMap;

use super::tgp::{COMPS, Comp, Param, TgpValue, StaticString, asStaticString, DATA_PARAM, NOP, IKnownObj, ExtendCtx, SomeVarsDef, Profile};

#[derive(Clone, Debug)]
pub struct Ctx {
    pub profile: &'static TgpValue,
    pub parent_param: Option<&'static Param>,
    pub path: StaticString,
    pub params: Rc<StdHashMap<StaticString, Rc<RTValue>>>,
    pub cached_params: Option<&'static StdHashMap<StaticString, TgpValue>>,
    pub data: Rc<RTValue>,
    pub vars: Rc<StdHashMap<StaticString, Rc<RTValue>>>,
    pub cmp_ctx: Option<Rc<Ctx>>,
    pub comps: &'static StdHashMap<StaticString, Comp>,
}

impl Ctx {
    // Constructor function with selective defaults
    pub fn new() -> Self {
        Ctx {profile: &NOP, data: Rc::new(RTValue::Null), vars: Rc::new(HashMap::new()), parent_param: None, path: "", 
            params: Rc::new(HashMap::new()), cmp_ctx: None,comps: &COMPS, cached_params: None }
    }
    pub fn set_data(&self, data: Rc<RTValue>) -> Self {
        Ctx { data, profile: self.profile, vars: Rc::clone(&self.vars), parent_param: self.parent_param, path: self.path, 
            params: Rc::clone(&self.params), cmp_ctx: self.cmp_ctx.as_ref().map(Rc::clone), comps: self.comps, cached_params: self.cached_params,
        }
    }
    pub fn set_profile(&self, profile: &'static TgpValue) -> Self {
        Ctx { data: Rc::clone(&self.data), profile, vars: Rc::clone(&self.vars), parent_param: self.parent_param, path: self.path, 
            params: Rc::clone(&self.params), cmp_ctx: self.cmp_ctx.as_ref().map(Rc::clone), comps: self.comps, cached_params: self.cached_params,
        }
    }
    pub fn profile_and_path(&self, profile: &'static TgpValue, parent_param: &'static Param, path: StaticString) -> Self {
        Ctx { profile, path , parent_param: Some(parent_param), 
            data: Rc::clone(&self.data), vars: Rc::clone(&self.vars), 
            params: Rc::clone(&self.params), cmp_ctx: self.cmp_ctx.as_ref().map(Rc::clone), comps: self.comps, cached_params: self.cached_params,
        }
    }    
    pub fn inner_profile(&self, profile: &Profile, parent_param: &'static Param) -> Self {
        let param_id = parent_param.id;
        self.profile_and_path(profile.props[param_id], parent_param, asStaticString(&format!("{}/{param_id}", self.path)))
    }
    pub fn inner_profile_in_array(&self, profile: &Profile, parent_param: &'static Param, index: usize) -> Self {
        let param_id = parent_param.id;
        self.profile_and_path(profile.props[param_id], parent_param, asStaticString(&format!("{}/{param_id}/{index}", self.path)))
    }
    pub fn new_comp(&self, params: HashMap<StaticString, Rc<RTValue>>, comp: &'static Comp) -> Self {
        let pt = comp.id;
        Ctx { 
            cmp_ctx: Some(Rc::new(self.clone())),
            params: Rc::new(params),
            path: asStaticString(&format!("{pt}/impl")),
            profile: &comp.r#impl, parent_param: self.parent_param,
            data: Rc::clone(&self.data), vars: Rc::clone(&self.vars), 
            comps: self.comps, cached_params: None,
        }
    }

}

#[derive(Clone, Debug)]
pub enum RTValue {
    Null,
    StaticString(StaticString),
    I32(i32),
    Boolean(bool),
    DynString(String),
    IntArray(Vec<i32>),
    StaticSringArray(Vec<StaticString>),
    Array(Vec<Rc<RTValue>>),
    Obj(StdHashMap<StaticString, Rc<RTValue>>),
    KnownObj(Arc<Box<dyn IKnownObj>>),
    Error(String),
    Ctx(Ctx)
}

type RTResult = (Ctx, RTValue);
trait Transformer {
    fn transform(&self, item: RTValue) -> RTResult;
}

pub fn jb_run(ctx: Ctx) -> RTResult {
    match ctx.profile {
        TgpValue::StaticString(s) => (ctx, RTValue::StaticString(*s)),
        TgpValue::String(s) => (ctx, RTValue::DynString(s.clone())),
        TgpValue::I32(n) => (ctx, RTValue::I32(*n)),
        TgpValue::Boolean(b) => (ctx, RTValue::Boolean(*b)),
        TgpValue::Array(_) => panic!("no run array"),
        TgpValue::ConstsOnlyProfile(const_profile) => {
            let pt = const_profile.pt;
            jb_run(ctx.set_profile(&ctx.comps.get(pt).unwrap().r#impl))
        },
        TgpValue::CompiledProfile(profile) => {
            let f: &Box<dyn IKnownObj> = profile.compiled;
            if let Some(transform_func) = f.query_interface("Transformer") {
                transform_func(ctx, None)
            } else {
                (ctx, RTValue::Error("err".to_string()))
            }
        },        
        TgpValue::Profile(profile, extend_ctx) => {
            let new_ctx = extend_ctx.map_or(ctx.clone(), |extend_ctx| { calc_extend_ctx(ctx, extend_ctx) });
            let pt = profile.pt;
            let comp = new_ctx.comps.get(pt).unwrap();
            let params: HashMap<StaticString, Rc<RTValue>> = comp.params.iter().map(|parent_param| {
                let param_id = parent_param.id;
                let inner_profile = &profile.props[param_id];
                if let TgpValue::Array(inner) = inner_profile {
                    (param_id, Rc::new(RTValue::Array(inner.iter().enumerate().map(|(i,_)| {
                        Rc::new(jb_run(new_ctx.inner_profile_in_array(profile, parent_param, i)).1)
                    }).collect())))
                } else {
                    (param_id, Rc::new(jb_run(new_ctx.inner_profile(profile, parent_param)).1))
                }
            }).collect();
            jb_run(new_ctx.new_comp(params, comp))
        }
        TgpValue::Nop() => {
            let new_data = (*ctx.data).clone();
            (ctx, new_data)
        },
        TgpValue::KnownObj(_) => todo!(),
    }
    //...
}

fn calc_extend_ctx(ctx: Ctx, extend_ctx: &'static ExtendCtx) -> Ctx {
    let new_ctx = ctx.clone();
    let new_vars = match extend_ctx.vars {
        Some(someVarsDef) => match someVarsDef {
            SomeVarsDef::VarsDef(vars ) => {
                let mut new_hash = (*ctx.vars).clone();
                for (i, var) in vars.iter().enumerate() {
                    new_hash.insert(var.0 , Rc::new(jb_run(
                        ctx.profile_and_path(var.1.unwrap_or(&NOP), &DATA_PARAM, asStaticString(&format!("{}/$vars/{i}", ctx.path)))
                    ).1));                            
                }
                new_hash
            },
            SomeVarsDef::VarDef(id, val ) => {
                let mut new_hash = (*ctx.vars).clone();
                new_hash.insert(id , Rc::new(jb_run(
                    ctx.profile_and_path(val.unwrap_or(&NOP), &DATA_PARAM, asStaticString(&format!("{}/$vars", ctx.path)))
                ).1));
                new_hash
            }
        },
        None => (*(ctx.vars)).clone(),
    };
    let new_data : Rc<RTValue> = match extend_ctx.data {
        Some(profile) => Rc::new(jb_run(ctx.profile_and_path(profile, &DATA_PARAM, asStaticString(&format!("{}/data", ctx.path)))).1),
        None => ctx.data,
    };
    Ctx { data: new_data, vars: Rc::new(new_vars), ..new_ctx }
}
