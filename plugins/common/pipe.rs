use ctor::ctor;
extern crate paste;
use paste::paste;

use crate::core::rt::{Ctx, RTObj, RTValue }; 
use crate::core::tgp::{Comp, CompsTrait, Param, Profile, RustImpl, StaticString, TgpValue, COMPS };
use maplit::hashmap;

use std::rc::Rc;
use std::sync::Arc;

trait StringOperator {
    fn apply(&self, s: &str) -> String;
}

fn apply_string_operator(ctx: &Ctx, operator: &dyn StringOperator) -> RTValue {
    let val: &RTValue = &ctx.data;
    match val {
        RTValue::Array(arr) => RTValue::Array(
            arr.iter()
                .map(|x| match x {
                    RTValue::StaticString(s) => RTValue::DynString(operator.apply(s)),
                    RTValue::DynString(s) => RTValue::DynString(operator.apply(s)),
                    _ => RTValue::Error("Non-string value encountered".to_string(), Some(ctx.clone())),
                })
                .collect(),
        ),
        RTValue::StaticStringArray(arr) => RTValue::Array(
            arr.iter()
                .map(|x| RTValue::DynString(operator.apply(x)))
                .collect(),
        ),
        RTValue::StaticString(s) => RTValue::DynString(operator.apply(s)),
        RTValue::DynString(s) => RTValue::DynString(operator.apply(s)),
        _ => RTValue::Error("Non-string value encountered".to_string(), Some(ctx.clone())),
    }
}

macro_rules! string_operator_comp {
    ($id: ident, $method:ident) => {{
        #[derive(Debug)]
        struct _StringOperator;
        use crate::core::tgp::{Comp, RustImpl, TgpValue, COMPS};
        use crate::core::rt::RTValue;
        use crate::common::pipe::{StringOperator, apply_string_operator};
        use std::sync::Arc;

        impl StringOperator for _StringOperator {
            fn apply(&self, s: &str) -> String {
                s.$method()
            }
        }
        impl RustImpl for _StringOperator {
            fn run(&self, ctx: &$crate::Ctx) -> RTValue {
                apply_string_operator(ctx, self)
            }
        }
        // let new_comp = Comp { id: "$id",  r#type: "data", params: vec![], r#impl: TgpValue::RustImpl(Arc::new(_StringOperator)) };
        // COMPS.add("$id", new_comp);
        // let x = COMPS.get("$id");
        Comp { id: "$id",  r#type: "data", params: vec![], r#impl: TgpValue::RustImpl(Arc::new(_StringOperator)) }
    }
  };
}

#[derive(Debug)]
struct Pipe;
impl RustImpl for Pipe {
    fn run(&self, ctx: &Ctx) -> RTValue {
        let source = ctx.calc_dynamic_param("source", None, None);
        match source {
            Some(val) => ctx.get_param("operator").unwrap().into_iter().fold(val, |agg, dynamic_script| 
                step(match dynamic_script { RTValue::Func(ctx) => ctx, _ => panic!("pipe: invalid operator param")} , agg)),    
            None => RTValue::Error("Pipe: Missing No source param".to_string(), Some(ctx.clone()))
        }
    }
}
fn step(dynamic_script: Ctx, data: RTValue) -> RTValue {
    match data {
        RTValue::Shared(rc_data) => dynamic_script.set_data(rc_data).run_itself(),
        _ => dynamic_script.set_data(Rc::new(data)).run_itself()
    }
}

macro_rules! component {
    ($id:ident, $params: expr, $func: item) => {
        #[derive(Debug)]
        struct $id;
        impl RustImpl for $id {
             $func
        }
        paste! {
            #[ctor]
            fn [<$id _init>]() {
                let _id = stringify!($id);
                COMPS.add(_id, Comp{
                    id: _id,
                    r#type: "data",
                    params: $params, 
                    r#impl: TgpValue::RustImpl(Arc::new($id)),
                });
            }
        }
    };
}

component!(pipe,
    vec!{
        Param{id: "source", r#type: Some("data"), default_value: None},
        Param{id: "operator", r#type: Some("data"), default_value: None},
    },
    fn run(&self, ctx: &Ctx) -> RTValue {
        let source = ctx.calc_dynamic_param("source", None, None);
        match source {
            Some(val) => ctx.get_param("operator").unwrap().into_iter().fold(val, |agg, _oper| {
                let operator_func = match _oper { RTValue::Func(ctx) => ctx, _ => panic!("pipe: invalid operator param")};
                match agg {
                    RTValue::Shared(rc_data) => operator_func.set_data(rc_data).run_itself(),
                    _ => operator_func.set_data(Rc::new(agg)).run_itself()
                }
            }),    
            None => RTValue::Error("Pipe: Missing No source param".to_string(), Some(ctx.clone()))
        }
    }
);

component!(split,
    vec!{
        Param {id: "separator", r#type: Some("data"), default_value: Some(&TgpValue::String(",")) }
    },
    fn run(&self, ctx: &Ctx) -> RTValue {
        let sep = ctx.get_string_param("separator");        
        match &*ctx.data {
            RTValue::StaticString(s) => RTValue::StaticStringArray(s.split(sep).collect()),
            RTValue::DynString(s) => RTValue::Array(s.split(sep).map(|s| RTValue::DynString(s.to_owned())).collect()),
            _ => RTValue::Error("split unsupported input type".to_string(), Some(ctx.clone()))
        }
});


#[ctor]
fn init() {
        COMPS.add("toUpperCase", string_operator_comp!(toUpperCase,to_uppercase));
        COMPS.add("splitTest", Comp {
            id: "splitTest",
            r#type: "data",
            params: vec![],         
            r#impl: Profile::new("split", hashmap!{"separator" => TgpValue::String("\n")}), 
        });
        COMPS.add("pipeTest", Comp {
            id: "pipeTest", 
            r#type: "data",
            params: vec![],         
            r#impl: Profile::new("pipe", hashmap!{
                "source" => TgpValue::String("a,b,c"),
                "operator" => TgpValue::Array(vec![
                    Profile::new("split", hashmap!{"separator" => TgpValue::String(",")}),
                    Profile::new("toUpperCase", hashmap!{})
                ])
            }) 
        });
}

#[cfg(ignore)]
component!("pipeTest", {
    type: "data",
    impl: pipe({source: "a,b,c", operator: [split(","), toUpperCase()]})
});