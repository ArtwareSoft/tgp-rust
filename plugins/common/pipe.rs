use ctor::ctor;
extern crate paste;
use paste::paste;
//use proc_macro2::{Ident, Literal, Punct, Token, TokenStream};

use crate::core::rt::{Ctx, RTValue }; 
use crate::core::tgp::{Comp, CompsTrait, Param, Profile, RustImpl, TgpValue, COMPS };
use maplit::hashmap;

use std::rc::Rc;
use std::sync::Arc;

macro_rules! rust_comp {
    ($id:ident, $params: expr, $func: item) => {
        #[derive(Debug)]
        #[allow(non_camel_case_types)]

        struct $id;
        impl RustImpl for $id {
             $func
        }
        paste! {
            #[ctor]
            #[allow(non_snake_case)]
            fn [<$id _init>]() {
                let _id = stringify!($id);
                COMPS.add(_id, Comp{
                    id: _id,
                    r#type: "data",
                    params: $params, 
                    r#impl: TgpValue::RustImpl(Arc::new($id)),
                    src: &TgpValue::Nop()
                });
            }
        }
    };
}

rust_comp!(pipe,
    vec!{
        Param{id: "source", r#type: "data", default_value: None, src: &TgpValue::Nop()},
        Param{id: "operator", r#type: "data", default_value: None, src: &TgpValue::Nop()},
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

rust_comp!(split,
    vec!{
        Param {id: "separator", r#type: "data", default_value: Some(&TgpValue::String(",")), src: &TgpValue::Nop() }
    },
    fn run(&self, ctx: &Ctx) -> RTValue {
        let sep = ctx.get_string_param("separator");        
        match &*ctx.data {
            RTValue::StaticString(s) => RTValue::StaticStringArray(s.split(sep).collect()),
            RTValue::DynString(s) => RTValue::Array(s.split(sep).map(|s| RTValue::DynString(s.to_owned())).collect()),
            _ => RTValue::Error("split unsupported input type".to_string(), Some(ctx.clone()))
        }
});

macro_rules! string_operator_comp {
    ($id: ident, $method:ident) => { rust_comp! {$id,
        vec!{},
        fn run(&self, ctx: &Ctx) -> RTValue {
            let val: &RTValue = &ctx.data;
            match val {
                RTValue::Array(arr) => RTValue::Array(arr.iter().map(|x| match x {
                        RTValue::StaticString(s) => RTValue::DynString(s.$method()),
                        RTValue::DynString(s) => RTValue::DynString(s.$method()),
                        _ => RTValue::Error("$id Non-string value encountered".to_string(), Some(ctx.clone())),
                    }).collect(),
                ),
                RTValue::StaticStringArray(arr) => RTValue::Array(arr.iter().map(|s| RTValue::DynString(s.$method())).collect()),
                RTValue::StaticString(s) => RTValue::DynString(s.$method()),
                RTValue::DynString(s) => RTValue::DynString(s.$method()),
                _ => RTValue::Error("$id Non-string value encountered".to_string(), Some(ctx.clone())),
            }            
        }
     }
    }
}

string_operator_comp!(toUpperCase,to_uppercase);

#[ctor]
fn init() {
        // COMPS.add("splitTest", Comp {
        //     id: "splitTest",
        //     r#type: "data",
        //     params: vec![],         
        //     r#impl: Profile::new("split", hashmap!{"separator" => TgpValue::String("\n")}), 
        // });
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
            }),
            src: &TgpValue::Nop()
        });
}

#[cfg(ignore)]
rust_comp!("pipeTest", {
    type: "data",
    impl: pipe({source: "a,b,c", operator: [split(","), toUpperCase()]})
});

