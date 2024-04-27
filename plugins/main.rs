use core::{rt::{Ctx}, tgp::{COMPS, CompsTrait }};

mod core;
mod common;


fn main() {
    {
        if let Some(comp) = COMPS.get("pipeTest") {
            println!("{:#?}", Ctx::new().set_profile(&comp.r#impl).run_itself());
        } else {
            println!("Component not found");
        }
    }
}


