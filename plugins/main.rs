use core::{rt::{Ctx}, tgp::{COMPS, CompsTrait }};

mod core;
mod common;
use std::env;

use crate::core::tgp::TgpValue;

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
    let json_text2 = r#"{
        "$$": "pipe",
        "source": "a,b,c,d",
        "operator": [ {"$$" : "split", "separator": ","}, {"$$" : "toUpperCase"} ]
    }"#;    
    println!("{:#?}", Ctx::new().set_profile(TgpValue::parse_json(json_text)).run_itself());
    //println!("{:#?}", Ctx::new().set_profile(TgpValue::parse_json(json_text2)).run_itself());
}
