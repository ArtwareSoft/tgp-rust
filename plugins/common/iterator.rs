use crate::core::rt::{Ctx, RTObj, RTValue }; 
use crate::core::tgp::StaticString;


impl IntoIterator for RTValue {
    type Item = RTValue;
    type IntoIter = RTIter;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            RTValue::Null => RTIter::Empty,
            RTValue::StaticString(s) => RTIter::StaticString(Some(s)),
            RTValue::I32(i) => RTIter::I32(Some(i)),
            RTValue::F64(i) => RTIter::F64(Some(i)),
            RTValue::Boolean(b) => RTIter::Boolean(Some(b)),
            RTValue::DynString(s) => RTIter::DynString(Some(s)),
            RTValue::IntArray(arr) => RTIter::IntArray(arr.into_iter()),
            RTValue::StaticStringArray(arr) => RTIter::StaticStringArray(arr.into_iter()),
            RTValue::Shared(x) => RTValue::clone(&x).into_iter(),
            RTValue::Obj(x) => RTIter::Obj(Some(x)),
            RTValue::Array(arr) => RTIter::Array(arr.into_iter()),
            RTValue::Error(err, ctx_opt) => RTIter::Error(Some(err), ctx_opt),
            RTValue::Func(ctx) => RTIter::Func(Some(ctx)),
        }
    }
}
pub enum RTIter {
    Empty,
    StaticString(Option<StaticString>),
    I32(Option<i32>),
    F64(Option<f64>),
    Boolean(Option<bool>),
    DynString(Option<String>),
    IntArray(std::vec::IntoIter<i32>),
    StaticStringArray(std::vec::IntoIter<StaticString>),
    Array(std::vec::IntoIter<RTValue>),
    Obj(Option<RTObj>),
    Error(Option<String>,Option<Ctx>),
    Func(Option<Ctx>),
}

impl Iterator for RTIter {
    type Item = RTValue;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            RTIter::Empty => None,
            RTIter::StaticString(opt) => opt.take().map(RTValue::StaticString),
            RTIter::I32(opt) => opt.take().map(RTValue::I32),
            RTIter::F64(opt) => opt.take().map(RTValue::F64),
            RTIter::Boolean(opt) => opt.take().map(RTValue::Boolean),
            RTIter::DynString(opt) => opt.take().map(RTValue::DynString),
            RTIter::IntArray(iter) => iter.next().map(RTValue::I32),
            RTIter::StaticStringArray(iter) => iter.next().map(|x| RTValue::StaticString(x)),
            RTIter::Array(iter) => iter.next(),
            RTIter::Error(opt, ctx_opt) => opt.take().map(|x| RTValue::Error(x, ctx_opt.clone())),
            RTIter::Func(opt) => opt.take().map(RTValue::Func),
            RTIter::Obj(opt) => opt.take().map(RTValue::Obj),
        }
    }
}
