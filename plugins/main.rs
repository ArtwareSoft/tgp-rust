use core::{rt::{Ctx, RTValue, jb_run}, tgp::{COMPS, CompsTrait }};
use std::rc::Rc;

mod core;
mod common;


fn main() {
    {
        if let Some(comp) = COMPS.get("splitTest") {
            let input = RTValue::StaticString("Line one\nLine two\nLine three");
            let res = jb_run(Ctx::new().set_data(Rc::new(input)).set_profile(&comp.r#impl));
            println!("{:#?}", res);
        } else {
            println!("Component not found");
        }
    }
}


