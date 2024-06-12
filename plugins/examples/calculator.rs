use crate::core::comp::{COMPS};
use crate::core::tgp::{Ctx, FuncType, Profile, StaticString, TgpType, TgpValue };

use std::sync::Arc;
use tgp_macro::{comp};
use ctor::ctor;


pub struct Exp;
impl TgpType for Exp {
    type ResType = f64;
    fn from_ctx(ctx: &Arc<Ctx>) -> Self::ResType {
        match ctx.profile {
            TgpValue::Int(i) => (*i) as Self::ResType,
            TgpValue::Float(f) => (*f) as Self::ResType,
            TgpValue::Profile(_profile) => ctx.run::<Self>(),
            TgpValue::Iden(_iden) => ctx.run::<Self>(),
            _ => panic!("exp invalid expression {:?}", ctx.profile)
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

comp!(mul2, {
    type: Exp,
    params: [ 
        param(x, Exp), 
    ],
    impl: plus(x,x),
});

comp!(plus_test, {
    type: Exp,
    impl: plus(1,2)
});

comp!(mul2_test, {
    type: Exp,
    impl: mul2(5)
});


#[ctor]
fn init() {
    let x = TgpValue::RustImpl(Arc::new(Arc::new(| ctx : &Arc<Ctx> | {
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
add param usage (mul2)
add profile array support
build data flow dsl

*/