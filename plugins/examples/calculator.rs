use crate::core::comp::{COMPS};
use crate::core::tester::{Test, TestResult};
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
}

comp!(rawCode, {
    type: Exp,
    params: [ 
        param(x, Exp), 
        param(y, Exp) 
    ],
    impl: TgpValue::RustImpl(Arc::new(Arc::new(| ctx : &Arc<Ctx> | {
        match( ctx.prop::<Exp>("x"), ctx.prop::< Exp >("y")) 
        { 
            (x, y) => { x + y } 
        }
    }
    ) as FuncType < Exp >)),
});

comp!(plus, {
    type: Exp,
    params: [ 
        param(x, Exp), 
        param(y, Exp) 
    ],
    impl: fn (x: fn Exp, y: Exp) -> Exp { x() + y },
});


comp!(ptByExample, {
    type: Exp,
    params: [ 
        param(x, Exp), 
    ],
    impl: plus(x,x),
});

comp!(expTest, {
    type: Test,
    params: [ 
        param(res, Exp), 
        param(expectedResult, Exp),
    ],    
    impl: fn (res: Exp, expectedResult: Exp) -> Test { TestResult { 
        success: res == expectedResult, 
        test_id: ctx.comp.unwrap().id, 
        reason: format!("{} == {}", res, expectedResult)
    } }
});

comp!(plus_test, {
    type: Test,
    impl: expTest(plus(1,2), 3)
});

comp!(rawCode_test, {
    type: Test,
    impl: expTest(rawCode(1,2),3)
});

comp!(ptByExample_test, {
    type: Test,
    impl: expTest(ptByExample(5),10)
});


#[ctor]
fn init() {
}
