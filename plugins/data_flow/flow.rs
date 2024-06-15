use crate::core::comp::{COMPS};
use crate::core::tester::{Test, TestResult};
use crate::core::tgp::{Ctx, FuncType, Profile, StaticString, TgpType, TgpValue };

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use tgp_macro::{comp};
use ctor::ctor;

#[derive(Clone, Debug)]
pub enum Val {
    Null,
    StaticString(StaticString),
    Int(usize),
    Float(f64),
    Boolean(bool),
    DynString(String),
    IntArray(Vec<usize>),
    StaticStringArray(Vec<StaticString>),
    Shared(Arc<Val>),
    Array(Vec<Val>),
    Obj(DataObj),
}
pub type DataObj = HashMap<StaticString, Val>;
impl Val {
    fn Int(&self) -> usize { match self {
        Val::Int(i) => *i,
        _ => panic!("not an integer")
    }}
}

#[derive(Clone, Debug)]
pub struct FlowCtx {
    pub data: Option<Val>,
    pub vars: DataObj,
}
impl FlowCtx {
    fn new() -> FlowCtx { FlowCtx {data: None, vars: HashMap::new() } }
}

pub type FlowFuncType = Arc<dyn Fn(&FlowCtx) -> Val + Sync + Send>;

pub struct DataFuncs {
    calc: Option<FlowFuncType>,
    agg: Option<FlowFuncType>
}

pub struct Data;
impl TgpType for Data {
    type ResType = DataFuncs;
    fn from_ctx(ctx: &Arc<Ctx>) -> Self::ResType {
        match ctx.profile {
            TgpValue::Profile(_profile) => ctx.run::<Self>(),
            TgpValue::Iden(_iden) => ctx.run::<Self>(),
            TgpValue::Int(n) => DataFuncs {
                calc: Some(Arc::new( move|_: &FlowCtx| -> Val {Val::Int(*n)})),
                agg: Some(Arc::new( move|_: &FlowCtx| -> Val {Val::IntArray(vec!(*n))})),
            },
            _ => panic!("can not cast profile to DataFunc {:?}", ctx.profile)
        }
    }
}

comp!(dataTest, {
    type: Test,
    params: [ 
        param(calc, Data), 
        param(expectedResult, Data),
    ],    
    impl: fn (calc: Data, expectedResult: Data) -> Test { 
        let flow_ctx = FlowCtx::new();
        match (calc.calc.unwrap()(&flow_ctx).Int(), expectedResult.calc.unwrap()(&flow_ctx).Int()) {
            (res, expectedResult) => {
                TestResult { 
                    success: res == expectedResult, 
                    test_id: ctx.comp.unwrap().id, 
                    failure_reason: if res == expectedResult { None } else{ Some(format!("{} != {}", res, expectedResult)) }
                }
            }
        }
    }
});

comp!(plus, {
    type: Data,
    params: [ 
        param(x, Data), 
        param(y, Data) 
    ],
    impl: TgpValue::RustImpl(Arc::new(Arc::new( move | ctx : &Arc<Ctx> | {
        match( ctx.prop::<Data>("x").calc.unwrap(), ctx.prop::< Data >("y").calc.unwrap()) { 
            (x, y) => {
                DataFuncs {
                    calc: Some(Arc::new( move|fCtx: &FlowCtx| -> Val { Val::Int(
                        match (x(fCtx).Int(), y(fCtx).Int()) {
                            (x, y) => { x + y } 
                        }
                    )})),
                    agg: None,
                }
            }
        }
    }
    ) as FuncType < Data >)),
});


comp!(data_const_test, {
    type: Test,
    impl: dataTest(plus(1,2), 3)
});

#[ctor]
fn init_flow() {
}

/*
todo:

*/