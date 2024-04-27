use std::{collections::HashMap as StdHashMap, rc::Rc};
use std::collections::HashMap;

use super::tgp::{COMPS, Comp, Param, TgpValue, StaticString, asStaticString, DATA_PARAM, NOP, ExtendCtx, SomeVarsDef, Profile, CompsTrait };

pub type Data = Rc<RTValue>;
pub type Vars = Rc<StdHashMap<StaticString, RTValue>>;
pub type RTObj = StdHashMap<StaticString, RTValue>;

#[derive(Clone, Debug)]
pub struct Ctx {
    pub profile: &'static TgpValue,
    pub parent_param: Option<&'static Param>,
    pub path: StaticString,
    pub params: Rc<StdHashMap<StaticString, RTValue>>,
    pub data: Data,
    pub vars: Vars,
    pub cmp_ctx: Option<Rc<Ctx>>,
}

impl Ctx {
    pub fn new() -> Self {
        Ctx {profile: &NOP, data: Rc::new(RTValue::Null), vars: Rc::new(HashMap::new()), parent_param: None, path: "", 
            params: Rc::new(HashMap::new()), cmp_ctx: None }
    }
    pub fn set_data(&self, data: Rc<RTValue>) -> Self {
        Ctx { data, profile: self.profile, vars: Rc::clone(&self.vars), parent_param: self.parent_param, path: self.path, 
            params: Rc::clone(&self.params), cmp_ctx: self.cmp_ctx.as_ref().map(Rc::clone),
        }
    }
    pub fn set_vars(&self, vars: Vars) -> Self {
        Ctx { vars, 
            profile: self.profile, data: Rc::clone(&self.data), parent_param: self.parent_param, path: self.path, 
            params: Rc::clone(&self.params), cmp_ctx: self.cmp_ctx.as_ref().map(Rc::clone),
        }
    }
    pub fn set_profile(&self, profile: &'static TgpValue) -> Self {
        Ctx { data: Rc::clone(&self.data), profile, vars: Rc::clone(&self.vars), parent_param: self.parent_param, path: self.path, 
            params: Rc::clone(&self.params), cmp_ctx: self.cmp_ctx.as_ref().map(Rc::clone),
        }
    }
    pub fn profile_and_path(&self, profile: &'static TgpValue, parent_param: &'static Param, path: StaticString) -> Self {
        Ctx { profile, path , parent_param: Some(parent_param), 
            data: Rc::clone(&self.data), vars: Rc::clone(&self.vars), 
            params: Rc::clone(&self.params), cmp_ctx: self.cmp_ctx.as_ref().map(Rc::clone),
        }
    }    
    pub fn inner_profile(&self, profile: &'static Profile, parent_param: &'static Param) -> Self {
        let param_id = parent_param.id;
        self.profile_and_path(&profile.props[param_id], parent_param, asStaticString(&format!("{}/{param_id}", self.path)))
    }
    pub fn inner_profile_in_array(&self, inner_profile: &'static TgpValue, parent_param: &'static Param, index: usize) -> Self {
        let param_id = parent_param.id;
        self.profile_and_path(inner_profile, parent_param, asStaticString(&format!("{}/{param_id}/{index}", self.path)))
    }
    pub fn new_comp(&self, params: HashMap<StaticString, RTValue>, comp: &'static Comp) -> Self {
        let pt = comp.id;
        Ctx { 
            cmp_ctx: Some(Rc::new(self.clone())),
            params: Rc::new(params),
            path: asStaticString(&format!("{pt}/impl")),
            profile: &comp.r#impl, parent_param: self.parent_param,
            data: Rc::clone(&self.data), vars: Rc::clone(&self.vars), 
        }
    }
    pub fn get_param(&self, param_id: &str) -> Option<RTValue> {
        self.params.get(param_id).map_or(None, |v| Some(v.clone()))
    }
    pub fn calc_dynamic_param(&self, param_id: &str, data: Option<Data>, vars : Option<Vars>) -> Option<RTValue> {
        self.params.get(param_id).map_or(None, |p_value| {
            let p_val : &RTValue = &p_value;
            match p_val {
                RTValue::DynamicScript(run_ctx) => Some(jb_run(match (data, vars) {
                    (None, None) => run_ctx.clone(),
                    (None, Some(vars)) => run_ctx.set_vars(vars),
                    (Some(data), None) => run_ctx.set_data(data),
                    (Some(data), Some(vars)) => run_ctx.set_data(data).set_vars(vars),
                })),
                _ => Some(p_val.clone())
            }
        })
    }

    pub fn run_itself(&self) -> RTValue {
        jb_run(self.clone())
    }
    pub fn get_comp_param(&self, param_id: &str) -> Option<RTValue> {
        self.cmp_ctx.as_ref().map_or(None, |cmp_ctx| cmp_ctx.clone().params.clone().get(param_id).map_or(None, |v| Some(v.clone())) )
    }
    pub fn extend(&self, extend_ctx: &'static ExtendCtx) -> Ctx {
        let data_ctx = match extend_ctx.data {
            Some(profile) => self.set_data(Rc::new(jb_run(self.profile_and_path(profile, &DATA_PARAM, asStaticString(&format!("{}/data", self.path)))))),
            None => self.clone(),
        };

        let vars_ctx = match extend_ctx.vars {
            Some(someVarsDef) => match someVarsDef {
                SomeVarsDef::VarsDef(vars ) => {
                    let mut new_hash = (*data_ctx.vars).clone();
                    for (i, var) in vars.iter().enumerate() {
                        new_hash.insert(var.0 , jb_run(
                            data_ctx.profile_and_path(var.1.unwrap_or(&NOP), &DATA_PARAM, asStaticString(&format!("{}/$vars/{i}", data_ctx.path)))
                        ));                            
                    }
                    data_ctx.set_vars(Rc::new(new_hash))
                },
                SomeVarsDef::VarDef(id, val ) => {
                    let mut new_hash = (*data_ctx.vars).clone();
                    new_hash.insert(id , jb_run(
                        data_ctx.profile_and_path(val.unwrap_or(&NOP), &DATA_PARAM, asStaticString(&format!("{}/$vars", data_ctx.path)))
                    ));
                    data_ctx.set_vars(Rc::new(new_hash))
                }
            },
            None => data_ctx,
        };
        vars_ctx
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
    StaticStringArray(Vec<StaticString>),
    EmptyArray(),
    Shared(Rc<RTValue>),
    Array(Vec<RTValue>),
    Obj(RTObj),
    Error(String),
    DynamicScript(Ctx),
    DynamicScripts(Vec<Ctx>)
}


pub fn jb_run(ctx: Ctx) -> RTValue {
    match ctx.profile {
        TgpValue::StaticString(s) => RTValue::StaticString(*s),
        TgpValue::String(s) => RTValue::DynString(s.clone()),
        TgpValue::I32(n) => RTValue::I32(*n),
        TgpValue::Boolean(b) => RTValue::Boolean(*b),
        TgpValue::Array(_) => panic!("no run array"),
        TgpValue::ConstsOnlyProfile(const_profile) => {
            let pt = const_profile.pt;
            jb_run(ctx.set_profile(&COMPS.get(pt).unwrap().r#impl))
        },
        TgpValue::RustImpl(profile) => profile.run(&ctx),        
        TgpValue::Profile(profile, extend_ctx) => {
            let new_ctx = extend_ctx.map_or(ctx.clone(), |extend_ctx| ctx.extend(extend_ctx));
            let pt = profile.pt;
            let comp = COMPS.get(pt).unwrap();
            let params: HashMap<StaticString, RTValue> = comp.params.iter().map(|parent_param| {
                let param_id = parent_param.id;
                match &profile.props[param_id] {
                    TgpValue::Array(inner_array) => (param_id, RTValue::DynamicScripts(inner_array.iter().enumerate()
                      .map(|(i,inner_profile)| 
                          new_ctx.inner_profile_in_array(inner_profile, parent_param, i)).collect())),
                    _ => (param_id, RTValue::DynamicScript(new_ctx.inner_profile(profile, parent_param)))
                }
            }).collect();
            jb_run(new_ctx.new_comp(params, comp))
        }
        TgpValue::Nop() => (*ctx.data).clone(),
    }
}

