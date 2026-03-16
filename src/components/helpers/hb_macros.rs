use handlebars::handlebars_helper;

handlebars_helper!(stringify: |v: Json| {
    v.to_string()
});

handlebars_helper!(gt: |a: Json, b: Json| {
    let a_num = a.as_f64().unwrap_or(0.0);
    let b_num = b.as_f64().unwrap_or(0.0);
    a_num > b_num
});
