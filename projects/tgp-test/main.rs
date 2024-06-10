use tgp::core::rt::{Ctx, RTValue }; 
use tgp::core::comp::{COMPS, Comp, Param, as_static, DATA_PARAM, NOP, ParamFuncType };
use tgp::core::tgp::{TgpValue, StaticString, ExtendCtx, SomeVarsDef, Profile, TgpType };
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use tgp::examples::calculator::Exp;

use ctor::ctor;
use tgp_macro::{tgp_value,tgp_value_from_string,comp};
use lazy_static::lazy_static;

// fn main() {
// //    let args: Vec<String> = env::args().collect();

// //    println!("Number of arguments: {}", args.len());
// //    println!("Arguments: {:?}", args);

//     // Rest of your code...
// }

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
    let res = Exp::from_tgp_value(COMPS.get_impl("Exp<>plus_test").unwrap());
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

#[ctor]
fn init1() {
    //literal_value!(dsfsdf);
    //println!("{}", "Hello");
}

// comp!(a, {
//     params: [ 
//         Param(aa, { type: "hello[]", defaultValue: myProf(aa,3)}),
//     ],
//     impl: () => {}
// });

#[ctor]
fn init2() {
//  println!("{:?}", tgp_value!(a(5, {a : a, b: 3})));
//  println!("{:?}", tgp_value!(a(5, {a : a, b: 3})));

//     println!("{:?}", tgp_value!(jbComp {
//         id: "pipe",  elems: [3],
//     }
// ));
    // println!("{:?}", tgp_value_from_string!(r#"component(a, {
    //     params: [ { id: 'aa'}],
    //     impl: 5
    // })"#));
    // println!("{:?}", comp!(a, {
    //     type: "myType<myDsl>[]",
    //     params: [ 
    //         param(aa, {type: array(hello), defaultValue: myProf("aa",3)}),
    //         { id: vv, type: "hello[]", defaultValue: myProf("aa",3)},
    //     ],
    //     impl: fn run(&self, ctx: &Ctx) -> RTValue { RTValue::Int(5) }
    // }));
    // println!("{:?}", tgp_value_from_string!(r#"component('a', {
    //     impl: dataTest(pipeline(Var('a', 1), list('%$a%',2), join()), equals('1,2'))
    //   })"#));
    //println!("{:?}", tgp_value! ( dataTest { calc: 5, expectedResult: equals {to : 5 }} ));
}

// component!( pipeTest, { 
//     type: "data",
//     impl: pipe {source: "a,b,c", operator: [split(","), toUpperCase()]}
// });

// #[ctor]
// fn init() {
//     print!("{}", to_tgp_value! ( dataTest { calc: 5, expectedResult: equals {to : 5 }} ));
// }