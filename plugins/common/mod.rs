use ctor::ctor;

use crate::core::rt::{Ctx, RTObj, RTValue }; 
use crate::core::tgp::{Comp, CompsTrait, Param, Profile, RustImpl, StaticString, TgpValue, COMPS };
use maplit::hashmap;

use std::collections::HashMap as StdHashMap;
use std::rc::Rc;
use std::sync::Arc;

impl IntoIterator for RTValue {
    type Item = RTValue;
    type IntoIter = RTIter;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            RTValue::Null => RTIter::Empty,
            RTValue::StaticString(s) => RTIter::StaticString(Some(s)),
            RTValue::I32(i) => RTIter::I32(Some(i)),
            RTValue::Boolean(b) => RTIter::Boolean(Some(b)),
            RTValue::DynString(s) => RTIter::DynString(Some(s)),
            RTValue::IntArray(arr) => RTIter::IntArray(arr.into_iter()),
            RTValue::StaticStringArray(arr) => RTIter::StaticStringArray(arr.into_iter()),
            RTValue::EmptyArray() => RTIter::Empty,
            RTValue::Shared(_x) => RTIter::Empty,
            RTValue::Obj(x) => RTIter::Obj(Some(x)),
            RTValue::Array(arr) => RTIter::Array(arr.into_iter()),
            RTValue::Error(err) => RTIter::Error(Some(err)),
            RTValue::DynamicScript(ctx) => RTIter::DynamicScript(Some(ctx)),
            RTValue::DynamicScripts(ctxs) => RTIter::DynamicScripts(ctxs.into_iter()),
        }
    }
}
pub enum RTIter {
    Empty,
    StaticString(Option<StaticString>),
    I32(Option<i32>),
    Boolean(Option<bool>),
    DynString(Option<String>),
    IntArray(std::vec::IntoIter<i32>),
    StaticStringArray(std::vec::IntoIter<StaticString>),
    Array(std::vec::IntoIter<RTValue>),
    Obj(Option<RTObj>),
    Error(Option<String>),
    DynamicScript(Option<Ctx>),
    DynamicScripts(std::vec::IntoIter<Ctx>),
}

impl Iterator for RTIter {
    type Item = RTValue;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            RTIter::Empty => None,
            RTIter::StaticString(opt) => opt.take().map(RTValue::StaticString),
            RTIter::I32(opt) => opt.take().map(RTValue::I32),
            RTIter::Boolean(opt) => opt.take().map(RTValue::Boolean),
            RTIter::DynString(opt) => opt.take().map(RTValue::DynString),
            RTIter::IntArray(iter) => iter.next().map(RTValue::I32),
            RTIter::StaticStringArray(iter) => iter.next().map(|x| RTValue::StaticString(x)),
            RTIter::Array(iter) => iter.next(),
            RTIter::DynamicScripts(iter) => iter.next().map(|x| RTValue::DynamicScript(x) ),
            RTIter::Error(opt) => opt.take().map(RTValue::Error),
            RTIter::DynamicScript(opt) => opt.take().map(RTValue::DynamicScript),
            RTIter::Obj(opt) => opt.take().map(RTValue::Obj),
        }
    }
}

struct Pipe;
impl RustImpl for Pipe {
    fn run(&self, ctx: &Ctx) -> RTValue {
        let source = ctx.calc_dynamic_param("source", None, None);
        match source {
            Some(val) => {
             let x = ctx.get_param("operator").unwrap();
             let iter = x.into_iter();
             iter.fold(val, |agg, dynamic_script| 
                step(match dynamic_script { RTValue::DynamicScript(ctx) => ctx, _ => panic!("pipe: invalid operator param")} , agg))
             },    
            None => RTValue::Error("Pipe: Missing No source param".to_string())
        }
    }
    fn debug_info(&self) -> String { "Pipe".to_string() }
}
fn step(dynamic_script: Ctx, data: RTValue) -> RTValue {
    match data {
        RTValue::Shared(rc_data) => dynamic_script.set_data(rc_data).run_itself(),
        _ => dynamic_script.set_data(Rc::new(data)).run_itself()
    }
}

struct ToUpper;
impl RustImpl for ToUpper {
    fn run(&self, ctx: &Ctx) -> RTValue {
        let val : &RTValue = &ctx.data;
        match val {
            RTValue::Array(arr) => RTValue::Array(arr.into_iter().map(|x| match x {
                RTValue::StaticString(s) => RTValue::DynString(s.to_uppercase()),
                RTValue::DynString(s) => RTValue::DynString(s.to_uppercase()),
                _ => RTValue::Error("ToUpper non string as input".to_string())
            }).collect()),
            RTValue::StaticStringArray(arr) => RTValue::Array(arr.into_iter().map(|x| RTValue::DynString(x.to_uppercase())).collect()),
            RTValue::StaticString(s) => RTValue::DynString(s.to_uppercase()),
            RTValue::DynString(s) => RTValue::DynString(s.to_uppercase()),
            _ => RTValue::Error("ToUpper non string as input".to_string())
        }
    }
    fn debug_info(&self) -> String { "Pipe".to_string() }
}

struct Split;
impl RustImpl for Split {
    fn run(&self, ctx: &Ctx) -> RTValue {
        let separator : RTValue = ctx.get_param("separator").map_or(RTValue::StaticString(","), |v| v.clone());
        match *ctx.data {
            RTValue::StaticString(s) => match separator {
                RTValue::StaticString(sep) => RTValue::StaticStringArray(s.split(sep).collect()),
                RTValue::DynString(sep) => RTValue::StaticStringArray(s.split(&sep).collect()),
                _ => RTValue::StaticStringArray(s.split(",").collect())
            },
            _ => RTValue::Error("Unsupported type for transformation".to_string())
        }
    }
    fn debug_info(&self) -> String { "Split".to_string() }
}

#[ctor]
fn init() { 
        COMPS.add("pipe", Comp{
            id: "pipe",
            r#type: "data",
            params: vec![
                Param{id: "source", dynamic: true, r#type: Some("data"), default_value: None},
                Param{id: "operator", dynamic: true, r#type: Some("data"), default_value: None},
            ], 
            r#impl: TgpValue::RustImpl(Arc::new(Pipe)), 
        });
        COMPS.add("split", Comp{
            id: "split",
            r#type: "data",
            params: vec![Param::new("separator")], 
            r#impl: TgpValue::RustImpl(Arc::new(Split)), 
        });
        COMPS.add("toUpper", Comp{
            id: "toUpper",
            r#type: "data",
            params: vec![], 
            r#impl: TgpValue::RustImpl(Arc::new(ToUpper)), 
        });        
        COMPS.add("splitTest", Comp {
            id: "splitTest",
            r#type: "data",
            params: vec![],         
            r#impl: Profile::new("split", hashmap!{"separator" => TgpValue::StaticString("\n")}), 
        });
        COMPS.add("pipeTest", Comp {
            id: "pipeTest", 
            r#type: "data",
            params: vec![],         
            r#impl:Profile::new("pipe", hashmap!{
                "source" => TgpValue::StaticString("a,b,c"),
                "operator" => TgpValue::Array(vec![
                    Profile::new("split", hashmap!{"separator" => TgpValue::StaticString(",")}),
                    Profile::new("toUpper", hashmap!{})
                ])
            }) 
        });
}
