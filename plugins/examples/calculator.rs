use crate::core::comp::{COMPS};
use crate::core::tgp::{Ctx, FuncType, Profile, StaticString, TgpType, TgpValue };

use std::sync::Arc;
use tgp_macro::{comp};
use ctor::ctor;


pub struct Exp;
impl TgpType for Exp {
    type ResType = f64;
    fn from_ctx(ctx: &Ctx) -> Self::ResType {
        match ctx.profile {
            TgpValue::Int(i) => (*i) as Self::ResType,
            TgpValue::Float(f) => (*f) as Self::ResType,
            TgpValue::Profile(_profile) => ctx.run::<Self>(),
            _ => panic!("invalid exp {:?}", ctx)
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
    impl: fn (x: Exp, y: Exp) -> Exp { x + y },
});

comp!(plus_test, {
    type: Exp,
    impl: plus(1,2)
});


#[ctor]
fn init() {
    let x = TgpValue::RustImpl(Arc::new(Arc::new(| ctx : &Ctx | {
        match( ctx.prop::<Exp>("x"), ctx.prop::< Exp >("y")) 
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