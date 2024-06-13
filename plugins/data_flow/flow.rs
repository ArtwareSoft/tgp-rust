use crate::core::comp::{COMPS};
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
    pub data: Val,
    pub vars: DataObj,
}

// pub trait DataFuncTrait: Any + Send + Sync {
//     fn calc(flow_ctx: &FlowCtx) -> Val;
//     fn agg(flow_ctx: &FlowCtx) -> Val;
// }

pub type FlowFuncType = Arc<dyn Fn(&FlowCtx) -> Val + Sync + Send>;

#[derive(Clone, Debug)]
enum DataFuncType {
    Static = 0, Agg, Single
}

pub struct Data;
impl TgpType for Data {
    type ResType = FlowFuncType;
    fn from_ctx(ctx: &Arc<Ctx>) -> Self::ResType {
        match ctx.profile {
            TgpValue::Profile(_profile) => ctx.run::<Self>(),
            TgpValue::Iden(_iden) => ctx.run::<Self>(),
            _ => DataFuncImp::new(DataFuncType::Static, ctx.profile)
        }
    }
}

comp!(plus, {
    type: Data,
    params: [ 
        param(x, Data), 
        param(y, Data) 
    ],
    impl: fn (x: Data, y: Data) -> Data { 
        let res : FlowFuncType = Arc::new( move |fCtx: &FlowCtx| { Val::Int(x(fCtx).as_int() + y(fCtx).as_int()) });
        res
    },
});

comp!(plus2, {
    type: Data,
    params: [ 
        param(x, Data), 
        param(y, Data) 
    ],
    impl: flowFn (x: Data as Int, y: Data as Int) -> Int { x + y }
});


#[ctor]
fn init_flow() {
    let x = TgpValue::RustImpl(Arc::new(Arc::new(| ctx : &Arc<Ctx> | {
        match( ctx.prop::<Data>("x"), ctx.prop::< Data >("y")) 
        { 
            (x, y) => {
                let res : FlowFuncType = Arc::new( move |fCtx: &FlowCtx| {
                    let data_res = match (x(fCtx).Int(), y(fCtx).Int()) {
                        (x, y) => { x + y } 
                    };
                    Val::Int(data_res)
                });
                res
            }
        }
    }
    ) as FuncType < Data >));
}

#[derive(Clone, Debug)]
pub struct DataType;
impl TgpType for DataType {
    type ResType = Val;
    fn from_ctx(ctx: &Arc<Ctx>) -> Self::ResType {
        match ctx.profile {
        }
    }
}

#[derive(Clone, Debug)]
pub struct DtCtx {
    pub expected_type: Val,
    pub vars: DataObj,
}

struct DataFuncImpX {
    t: DataFuncType,
    profile: &'static TgpValue,
    resolved_type: Option<DataType>
}
impl DataFuncImpX {
    fn new(t: DataFuncType, profile: &'static TgpValue) -> DataFuncImp {
        DataFuncImp { t, profile, resolved_type: None }
    }
    fn calc(&self) -> Val {
        match self.t {
            DataFuncType::Static => match self.profile {
                TgpValue::String(s) => Val::StaticString(*s),
                TgpValue::Int(n) => Val::Int(*n),
                TgpValue::Float(n) => Val::Float(*n),
                TgpValue::Boolean(b) => Val::Boolean(*b),
                TgpValue::Array(_) => todo!(""),
                TgpValue::Obj(_) => todo!(""),
                _ => panic!("tgp val is not supported as static {:?}", self.profile)
            }
            DataFuncType::Agg => todo!(),
            DataFuncType::Single => todo!(),
        }
    }
}
/*
todo:

*/