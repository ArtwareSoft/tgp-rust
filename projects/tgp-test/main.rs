use tgp::core::{rt::Ctx, tgp::TgpValue};
use ctor::ctor;
//use tgp_macro::tgp_value;

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
fn init() {
    //literal_value!(dsfsdf);
    println!("{}", "Hello");
}