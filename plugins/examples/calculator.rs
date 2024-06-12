use crate::core::comp::{COMPS};
use crate::core::tgp::{Profile, StaticString, TgpType, TgpValue, FuncType };

use std::sync::Arc;
use tgp_macro::{comp};
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
    impl: fn (x: fn Exp, y: Exp) -> Exp { x() + y },
});

comp!(plus_test, {
    type: Exp,
    impl: plus(1,2)
});


#[ctor]
fn init() {
    let x = TgpValue::RustImpl(Arc::new(Arc::new(| profile : & 'static Profile | {
        match( profile.prop::<Exp>("x"), profile.prop::< Exp >("y")) 
        { 
            (x, y) => { x + y } 
        }
    }
    ) as FuncType < Exp >));
}

// comp!(commonTest_join, {
//   impl: dataTest(pipeline(list(1,2), "%%", join()), equals("1,2"))
// });

/*
todo:
1. build data flow dsl
2. add ctx to interface - use ctx versus create new ctx.
3. build interpreter with ctx
*/