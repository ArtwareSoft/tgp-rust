use tgp::core::comp::{COMPS, Comp, Param, as_static, DATA_PARAM, NOP };
use tgp::core::tester::Test;
use tgp::core::tgp::{Ctx, Profile, StaticString, TgpType, TgpValue };
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use tgp::examples::calculator::Exp;

use ctor::ctor;
use tgp_macro::{tgp_value,tgp_value_from_string,comp};

fn main() {
    COMPS.resolve();
    println!("Comps resolved");
    test_calculator();
}

fn test_calculator() {
    let res = Ctx::run_profile_by_id::<Test>("Test<>data_const_test");
    println!("test res: {:?}",res);
}


