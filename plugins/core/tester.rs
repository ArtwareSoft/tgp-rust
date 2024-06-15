use crate::core::comp::{COMPS};
use crate::core::tgp::{Ctx, FuncType, Profile, StaticString, TgpType, TgpValue };

use std::sync::Arc;
use tgp_macro::{comp};
use ctor::ctor;

#[derive(Clone, Debug)]
pub struct TestResult {
    pub success: bool,
    pub test_id: &'static str,
    pub failure_reason: Option<String>,
}
pub struct Test;
impl TgpType for Test {
    type ResType = TestResult;
    fn from_ctx(ctx: &Arc<Ctx>) -> Self::ResType {
        match ctx.profile {
            TgpValue::Profile(_profile) => ctx.run::<Self>(),
            TgpValue::Iden(_iden) => ctx.run::<Self>(),
            _ => panic!("exp invalid expression {:?}", ctx.profile)
        }
    }
}
