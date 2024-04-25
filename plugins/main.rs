use core::{rt::{Ctx, RTValue}, tgp::{IKnownObj,  NOP, COMPS, StaticString}};
use std::{collections::HashMap, rc::Rc, sync::Arc};

mod core;

#[cfg(target_feature = "avx2")]
fn my_simd_function(input: &str) -> u32 {

}

struct SplitNLTransformer;
type RTFunc = fn(Ctx, Option<&str>) -> (Ctx, RTValue);

impl IKnownObj for SplitNLTransformer {
    fn query_interface(&self, interface: &str) -> Option<RTFunc> {
        match interface {
            "Transformer" => Some(Self::transform),
            _ => None
        }
    }
    
    fn debug_info(&self) -> String {
        "Transformer<>SplitNLTransformer".to_string()
    }
}

impl SplitNLTransformer {
    fn transform(ctx: Ctx, _method_name: Option<&str>) -> (Ctx, RTValue) {
        match *ctx.data {
            RTValue::StaticString(s) => {
                let lines: Vec<StaticString> = s.split('\n')
                             .collect();
                (ctx, RTValue::StaticSringArray(lines))
            },
            _ => (ctx, RTValue::Error("Unsupported type for transformation".to_string()))
        }
    }
}

fn main() {
    let transformer: Arc<Box<dyn IKnownObj>> = Arc::new(Box::new(SplitNLTransformer));
    let rt_value = RTValue::KnownObj(transformer);

    let input = RTValue::StaticString("Line one\nLine two\nLine three");

    let in_ctx = Ctx::new().set_data(Rc::new(input));
    if let RTValue::KnownObj(known_obj) = rt_value {
        if let Some(transform_func) = known_obj.query_interface("Transformer") {
            let (_, res) = transform_func(in_ctx, None); 
            println!("{:#?}", res);
        }        
    }
}
