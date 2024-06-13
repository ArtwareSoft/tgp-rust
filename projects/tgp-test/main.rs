use tgp::core::comp::{COMPS, Comp, Param, as_static, DATA_PARAM, NOP };
use tgp::core::tgp::{Ctx, ExtendCtx, Profile, SomeVarsDef, StaticString, TgpType, TgpValue };
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use tgp::examples::calculator::Exp;

use ctor::ctor;
use tgp_macro::{tgp_value,tgp_value_from_string,comp};

fn main() {
    // println!("start resolving");
    // pub type ParamFuncType = Arc<dyn Fn(&'static Profile) -> Param + Sync + Send>;

    // pub type BaseType = Arc<dyn Any>;
    // let f = |_profile: &'static Profile| -> Param { Param::simple("test1","type1",None) };
    // let any_arc: BaseType = Arc::new(Arc::new(f) as ParamFuncType);

    // let res = match any_arc.downcast_ref::<ParamFuncType>() {
    //     Some(f) => f( Box::leak(Box::new(Profile{props: HashMap::new(), pt: "" }))),
    //     None => panic!("can not cast param impl 1 {:?}", any_arc.type_id()),
    // };
    // println!("res {:?}", res);

    COMPS.resolve();
    println!("Comps resolved");
    test_calculator();
    
    //test_parse_json();
    // if let Some(comp) = COMPS.get("pipeTest") {
    //     println!("{:#?}", Ctx::new().set_profile(&comp.r#impl).run_itself());
    // } else {
    //     println!("Component not found");
    // }
}

fn test_calculator() {
    let res = Exp::from_ctx(&Ctx::new(COMPS.get_impl("Exp<>rawCode_test").unwrap()));
    println!("plus: {}",res);
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

