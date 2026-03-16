use handlebars::{BlockContext, PathAndJson};

#[allow(dead_code)]
pub fn create_block<'rc>(param: &'rc PathAndJson<'rc>) -> BlockContext<'rc> {
    let mut block = BlockContext::new();

    if let Some(new_path) = param.context_path() {
        *block.base_path_mut() = new_path.clone();
    } else {
        block.set_base_value(param.value().clone());
    }

    block
}
