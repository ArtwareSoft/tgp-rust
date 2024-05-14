use tgp::core::{rt::Ctx};
use tgp::core::tgp::{COMPS, Comp, Param, TgpValue, StaticString, as_static, DATA_PARAM, NOP, ExtendCtx, SomeVarsDef, Profile, CompsTrait };

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
    {
        test_parse_json();
        // if let Some(comp) = COMPS.get("pipeTest") {
        //     println!("{:#?}", Ctx::new().set_profile(&comp.r#impl).run_itself());
        // } else {
        //     println!("Component not found");
        // }
    }
}

fn test_parse_json() {
    let json_text = r#"{
        "$$": "pipe",
        "source": "a,b,c",
        "operator": [ {"$$" : "split" }, {"$$" : "toUpperCase"} ]
    }"#;
    let _json_text2 = r#"{
        "$$": "pipe",
        "source": "a,b,c,d",
        "operator": [ {"$$" : "split", "separator": ","}, {"$$" : "toUpperCase"} ]
    }"#;    
    println!("{:#?}", Ctx::new().set_profile(TgpValue::parse_json(json_text)).run_itself());
    //println!("{:#?}", Ctx::new().set_profile(TgpValue::parse_json(json_text2)).run_itself());
}

#[ctor]
fn init1() {
    //literal_value!(dsfsdf);
    //println!("{}", "Hello");
}

// lazy_static! {
//     static ref A: TgpValue = comp!("a", {
//         params: [ 
//             { id: "aa", type: "hello[]", defaultValue: myProf("aa",3)},
//             { id: "bb", type: "hello[]", defaultValue: myProf("aa",3)},
//         ],
//         impl: () => {}
//     });
// }

#[ctor]
fn init2() {
    println!("{:?}", tgp_value!(a(5, {a : a, b: 3})));
//     println!("{:?}", tgp_value!(jbComp {
//         id: "pipe",  elems: [3],
//     }
// ));
    // println!("{:?}", tgp_value_from_string!(r#"component('a', {
    //     params: [ { id: 'aa'}],
    //     impl: 5
    // })"#));
    // println!("{:?}", comp!("a", {
    //     params: [ 
    //         { id: "aa", type: "hello[]", defaultValue: myProf("aa",3)},
    //         { id: "bb", type: "hello[]", defaultValue: myProf("aa",3)},
    //     ],
    //     impl: 5
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