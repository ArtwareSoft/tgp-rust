use tgp::core::comp::{COMPS, Comp, Param, as_static, DATA_PARAM, NOP };
use tgp::core::tester::Test;
use tgp::core::tgp::{Ctx, Profile, StaticString, TgpType, TgpValue };
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use tgp::examples::calculator::Exp;

use ctor::ctor;
use tgp_macro::{comp};

fn main() {
    COMPS.resolve();
    println!("Comps resolved");
    test_calculator();
}

fn test_calculator() {
    println!("test res: {:?}", Ctx::run_profile_by_id::<Test>("Test<>plus_test"));
}

fn test_comp_json() {
    println!("test_comp_json");
    // let res = comp_json!(  plus, {
    //     type: Exp,
    //     params: [ 
    //         param(x, Exp), 
    //         param(y, Exp) 
    //     ],
    // });
    //let res = Ctx::run_profile_by_id::<Test>("Test<>data_const_test");
//    println!("test res: {:?}",res);
}

// fn test_parse_json() {
//     let json_text = r#"{
//         "$$": "pipe",
//         "source": "a,b,c",
//         "operator": [ {"$$" : "split" }, {"$$" : "toUpperCase"} ]
//     }"#;
//     let _json_text2 = r#"{
//         "$$": "pipe",
//         "source": "a,b,c,d",
//         "operator": [ {"$$" : "split", "separator": ","}, {"$$" : "toUpperCase"} ]
//     }"#;    
//     println!("{:#?}", Ctx::new().set_profile(TgpValue::parse_json(json_text)).run_itself());
//     //println!("{:#?}", Ctx::new().set_profile(TgpValue::parse_json(json_text2)).run_itself());
// }

#[ctor]
fn init() {
        // COMPS.add("splitTest", Comp {
        //     id: "splitTest",
        //     r#type: "data",
        //     params: vec![],         
        //     r#impl: Profile::new("split", hashmap!{"separator" => TgpValue::String("\n")}), 
        // });
        // COMPS.add("pipeTest", Comp {
        //     id: "pipeTest", 
        //     r#type: "data",
        //     params: vec![],         
        //     r#impl: Profile::new("pipe", hashmap!{
        //         "source" => TgpValue::String("a,b,c"),
        //         "operator" => TgpValue::Array(vec![
        //             Profile::new("split", hashmap!{"separator" => TgpValue::String(",")}),
        //             Profile::new("toUpperCase", hashmap!{})
        //         ])
        //     }),
        //     src: &TgpValue::Nop()
        // });
}