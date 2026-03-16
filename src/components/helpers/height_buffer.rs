use crate::components::parsing::xml::get_u16_value;
use handlebars::{
    BlockContext, BlockParams, Context, Handlebars, Helper, HelperDef, HelperResult, JsonValue,
    Output, RenderContext, RenderErrorReason, Renderable,
};

#[derive(Clone, Copy)]
pub struct HeightBufferHelper;

impl HelperDef for HeightBufferHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        r: &'reg Handlebars<'reg>,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let param = h
            .param(0)
            .ok_or(RenderErrorReason::ParamNotFoundForIndex("height_buffer", 0))?;

        if param.value().is_array() {
            let lines = param.value().as_array().expect("Param 0 value error");
            let ctx_data = ctx.data().as_object().expect("Context data error");
            let props = ctx_data
                .get("props")
                .expect("Value Unpack Error: props")
                .as_object()
                .expect("Value Get Error: props");

            let area = props
                .get("area")
                .expect("Value Unpack Error: area")
                .as_object()
                .expect("Value Get Error: area");

            let height_u64 = get_u16_value(area, "height");
            let buffered_lines = if lines.len() > height_u64 as usize {
                lines[lines.len() - height_u64 as usize..].to_vec()
            } else {
                lines.to_vec()
            };

            let mut block = BlockContext::new();
            block.set_base_value(JsonValue::Array(buffered_lines.clone()));

            if let Some(block_param) = h.block_param() {
                let mut params = BlockParams::new();
                params.add_value(block_param, JsonValue::Array(buffered_lines))?;
                block.set_block_params(params);
            }
            rc.push_block(block);
            if let Some(t) = h.template() {
                t.render(r, ctx, rc, out)?;
            }
            rc.pop_block();
            Ok(())
        } else {
            Err(RenderErrorReason::ParamTypeMismatchForName(
                "height_buffer",
                "0".to_string(),
                "array".to_string(),
            )
            .into())
        }
    }
}

pub static HEIGHT_BUFFER_HELPER: HeightBufferHelper = HeightBufferHelper;
