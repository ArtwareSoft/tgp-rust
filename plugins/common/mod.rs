use ctor::ctor;

use crate::core::rt::{Ctx, RTValue }; 
use crate::core::tgp::{TgpValue, Profile, RustImpl, COMPS, Comp, Param, CompsTrait };
use maplit::hashmap;

struct Split;
impl RustImpl for Split {    
    fn run(&self, ctx: &Ctx) -> RTValue {
        println!("a{:?}", ctx.get_param("separator"));
        let separator : RTValue = ctx.get_param("separator").map_or(RTValue::StaticString(","), |v| (*v).clone());
        println!("b{:?}", separator);

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
        COMPS.add("split", Comp{
            id: "split",
            r#type: "data",
            params: vec![Param::new("separator")], 
            r#impl: TgpValue::RustImpl(Box::new(Split)), 
        });
        COMPS.add("splitTest", Comp {
            id: "splitTest", 
            r#type: "data",
            params: vec![],         
            r#impl: TgpValue::Profile(Profile::new("split", hashmap!{"separator" => &TgpValue::StaticString("\n")}), None), 
        });
        COMPS.add("splitTest2", Comp {
            id: "splitTest2", 
            r#type: "data",
            params: vec![
                Param::new("separator")
            ],         
            r#impl: TgpValue::StaticString("asasa"), 
        });
}
