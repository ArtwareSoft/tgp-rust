use crate::core::comp::{COMPS, Param, as_static, DATA_PARAM, NOP };
use crate::core::tgp::{ExtendCtx, Profile, SomeVarsDef, StaticString, TgpType, TgpValue, FuncType };

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use tgp_macro::{tgp_value, comp};
use ctor::ctor;


pub struct Exp;
impl TgpType for Exp {
    type ResType = f64;
    fn from_tgp_value(profile: &'static TgpValue) -> Self::ResType {
        match profile {
            TgpValue::Int(i) => (*i) as Self::ResType,
            TgpValue::Float(f) => (*f) as Self::ResType,
            TgpValue::Profile(profile) => profile.calc::<Self>(),
            _ => panic!("invalid exp {:?}", profile)
        }
    }
    fn default_value() -> Self::ResType { 0.0 }
}

comp!(plus, {
    type: Exp,
    params: [ 
        param(x, Exp), 
        param(y, Exp) 
    ],
    /*
    impl: (x: FuncType<Exp>, y: Exp) {
        x() + y
    },
    => impl2: |profile: &'static Profile| {
        match (profile.prop::<Exp>("x"), profile.prop::<Exp>("y")) {
            (x, y) => {
                x + y
            }
        }
    },
*/
    impl: fn <Exp> |profile: &'static Profile| {
        profile.prop::<Exp>("x") + profile.prop::<Exp>("y")
    }
});

comp!(plus_test, {
    type: Exp,
    impl: plus(1,2)
});

// #[ctor]
// fn test2() {
//     let res  = Exp::from_tgp_value(COMPS.get_impl("plus_test").unwrap());
//     println!("plus: {}",res);
// }

comp!(commonTest_join, {
  impl: dataTest(pipeline(list(1,2), "%%", join()), equals("1,2"))
});

/*
todo:
1. make plus work with lambda impl
2. make plus work with (x: FuncType<Exp>, y: Exp)
3. build test pt
*/