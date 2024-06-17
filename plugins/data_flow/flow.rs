use crate::core::comp::{as_static, COMPS};
use crate::core::tester::{Test, TestResult};
use crate::core::tgp::{Ctx, FuncType, Profile, StaticString, TgpType, TgpValue };

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use tgp_macro::{comp};
use ctor::ctor;

pub type DataObj = HashMap<StaticString, Val>;
#[derive(PartialEq)]
pub enum DataTypeEnum {
    String, Int, Bool, Float, Obj, StringArray, IntArray, ObjArray, OtherArray, Empty
}
impl DataTypeEnum {
    fn isString(&self) -> bool { *self == DataTypeEnum::String || *self == DataTypeEnum::StringArray }
    fn isInt(&self) -> bool { *self == DataTypeEnum::Int || *self == DataTypeEnum::IntArray }
}

#[derive(Clone, Debug)]
pub enum Val {
    Null,
    StaticString(StaticString),
    Int(usize),
    Float(f64),
    Boolean(bool),
    DynString(String),
    IntArray(Arc<Vec<usize>>),
    StaticStringArray(Arc<Vec<StaticString>>),
    Array(Arc<Vec<Val>>),
    Obj(Arc<DataObj>),
}
impl Val {
    fn data_type(&self) -> DataTypeEnum { 
        match self {
            Val::StaticString(_) => DataTypeEnum::String,
            _ => DataTypeEnum::Empty
        }
    }
    fn Int(&self) -> usize { match self {
        Val::Int(i) => *i,
        _ => panic!("not an integer {:?}", self)
    }}
    fn String(&self) -> StaticString { match self {
        Val::StaticString(x) => x,
        Val::DynString(x) => as_static(x),
        _ => panic!("not a string {:?}", self)
    }}    
    pub fn IntIter(&self) -> ValIter<'_,usize> {
        match self {
            Val::IntArray(arr) => ValIter { vec: Some(arr), single_item: None, index: 0},
            // Val::Array(arr) => ValIter { inner: arr, index: 0},
            Val::Null => ValIter { vec: None, single_item: None, index: 0},
            Val::Int(x) => ValIter { vec: None, single_item: Some(x), index: 0},
            _ => panic!("val does not contains int {:?}", self)
        }
    }
    pub fn StringIter(&self) -> StaticStringIter<'_> {
        match self {
            Val::StaticStringArray(arr) => StaticStringIter { vec: Some(arr), single_item: None, index: 0},
            // Val::Array(arr) => ValIter { inner: arr, index: 0},
            Val::Null => StaticStringIter { vec: None, single_item: None, index: 0},
            Val::StaticString(x) => StaticStringIter { vec: None, single_item: Some(x), index: 0},
            Val::DynString(x) => StaticStringIter { vec: None, single_item: Some(&as_static(x.as_str())), index: 0},
            _ => panic!("val does not contains String {:?}", self)
        }
    }
}

struct ValIter<'a,T> {
    vec: Option<&'a Arc<Vec<T>>>,
    single_item: Option<&'a T>, 
    index: usize,
}
impl<'a, T> Iterator for ValIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.vec.is_some() { 
            let v = self.vec.unwrap();
            if self.index < v.len() {
                self.index += 1;
                Some(&v[self.index])
            } else {
                None
            }
        } else if self.single_item.is_some() { 
            if self.index < 1 {
                self.index += 1;
                Some(self.single_item.unwrap())
            } else {
                None
            }
        } else {
            None
        }
    }
}

struct StaticStringIter<'a> {
    vec: Option<&'a Arc<Vec<StaticString>>>,
    single_item: Option<StaticString>, 
    index: usize,
}
impl<'a> Iterator for StaticStringIter<'a> {
    type Item = StaticString;
    fn next(&mut self) -> Option<Self::Item> {
        if self.vec.is_some() { 
            let v = self.vec.unwrap();
            if self.index < v.len() {
                self.index += 1;
                Some(&v[self.index])
            } else {
                None
            }
        } else if self.single_item.is_some() { 
            if self.index < 1 {
                self.index += 1;
                Some(self.single_item.unwrap())
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct FlowCtx {
    pub data: Val,
    pub vars: Arc<DataObj>,
}
impl FlowCtx {
    fn new() -> FlowCtx { FlowCtx {data: Val::Null, vars: Arc::new(HashMap::new()) } }
    fn set_data(&self, val: Val) -> FlowCtx {
        FlowCtx {data: val, vars: self.vars.clone() }
    }
    fn set_vars(&self, new_vars: DataObj) -> FlowCtx {
        let mut vars: DataObj = HashMap::new();
        self.vars.iter().for_each(|(k,v)| { vars.insert(*k, v.clone()); });    
        new_vars.iter().for_each(|(k,v)| { vars.insert(*k, v.clone()); });
        FlowCtx {data: self.data.clone(), vars: Arc::new(vars) }
    }
    fn set_var(&self, id: StaticString, val: Val) -> FlowCtx {
        let mut vars: DataObj = HashMap::new();
        self.vars.iter().for_each(|(k,v)| { vars.insert(*k, v.clone()); });    
        vars.insert(id, val.clone());
        FlowCtx {data: self.data.clone(), vars: Arc::new(vars) }
    }
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
                agg: Some(Arc::new( move|_: &FlowCtx| -> Val {Val::IntArray(Arc::new(vec!(*n)))})),
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
                    reason: format!("{} == {}", res, expectedResult)
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
        DataFuncs {
            calc: match( ctx.prop::<Data>("x").calc.unwrap(), ctx.prop::<Data>("y").calc.unwrap()) { 
                    (x, y) => Some(Arc::new( move |fCtx: &FlowCtx| -> Val { Val::Int(
                        match (x(fCtx).Int(), y(fCtx).Int()) {
                            (x, y) => { x + y } 
                        }
                    )}))
                },
            agg: match( ctx.prop::<Data>("x").calc.unwrap(), ctx.prop::<Data>("y").calc.unwrap()) { 
                (x, y) => Some(Arc::new( move |fCtx: &FlowCtx| -> Val { Val::IntArray(
                    match (x(fCtx).Int(), y(fCtx).IntIter()) {
                        (x, y) => Arc::new(y.map(|y| x+y).collect())
                    }
                )}))
            }
        }
    }
    ) as FuncType < Data >)),
});

// comp!(pipe, {
//     type: Data,
//     params: [ 
//         param(source, Data), 
//         param(operators, "Data[]") 
//     ],
//     impl: TgpValue::RustImpl(Arc::new(Arc::new( move | ctx : &Arc<Ctx> | {
//         DataFuncs {
//             calc: match ( ctx.prop::<Data>("source").calc.unwrap(), ctx.prop_array::<Data>("operators").iter()) { 
//                     (source, operators) => Some(Arc::new( move |fCtx: &FlowCtx| -> Val { 
//                         operators.fold(source(fCtx), |acc: Val, op| {
//                             let input = fCtx.set_data(acc);
//                             if op.agg.is_some() { op.agg.unwrap()(&input) } else {
//                                 Val::Array(input.data.iter().map(|v| op.calc.unwrap()(v)).collect())
//                             }
//                         })
//                     }))
//                 },
//             agg: None
//         }
//     }
//     ) as FuncType < Data >)),
// });

comp!(data_const_test, {
    type: Test,
    impl: dataTest(plus(1,2), 3)
});

// comp!(data_const_test, {
//     type: Test,
//     impl: dataTest(plus(1,2), 3)
// });

#[ctor]
fn init_flow() {
}

/*
todo:
 pipe
 Condition
 Macro bootstrapping
 Tgppup of type

*/