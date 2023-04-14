use deno_core::v8;
use tegmine_common::prelude::*;

use super::Evaluator;
use crate::utils::DenoUtils;

pub struct JavascriptEvaluator;

thread_local! {
    static JS_RUNTIME : RefCell<Option<deno_core::JsRuntime>> = RefCell::default();
}

impl Evaluator for JavascriptEvaluator {
    fn evaluate(&self, expression: &InlineStr, input: &Object) -> TegResult<Object> {
        debug!("Javascript evaluator -- expression: {}", expression);
        // Evaluate the expression by using the Javascript evaluation engine.

        let func_name = format!("evaluate_{:x}", md5::compute(expression));
        let source_code = format!(
            "globalThis.{} = ($)=> {{ return {} }};",
            func_name, expression
        );

        let eval_result: TegResult<Object> = JS_RUNTIME.with(|js_runtime| {
            // at the first time, create JsRuntime
            if js_runtime.borrow().is_none() {
                *js_runtime.borrow_mut() = Some(new_js_runtime());
            }
            let mut guard = js_runtime.borrow_mut();
            let js_runtime = guard.as_mut().expect("not none");

            let func = get_global_func(js_runtime, &func_name).unwrap_or_else(|| {
                // at the first time, create function in JsRuntime
                set_global_func(
                    js_runtime,
                    Box::leak(func_name.clone().into_boxed_str()),
                    Box::leak(source_code.into_boxed_str()),
                );

                get_global_func(js_runtime, &func_name).expect("already set global func")
            });

            // call function to evaluate expression
            execute_global_func(js_runtime, func, input)
        });

        match eval_result {
            Ok(result) => {
                debug!("Javascript evaluator -- result: {:?}", result);
                Ok(result)
            }
            Err(e) => {
                error!(
                    "Error while evaluating script: {}, error: {:?}",
                    expression, e
                );
                fmt_err!(TerminateWorkflow, "{}", e)
            }
        }
    }
}

fn new_js_runtime() -> deno_core::JsRuntime {
    let js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
        ..Default::default()
    });
    js_runtime
}

fn get_global_func(
    js_runtime: &mut deno_core::JsRuntime,
    func_name: &str,
) -> Option<v8::Global<v8::Function>> {
    let context = js_runtime.global_context();
    let scope = &mut js_runtime.handle_scope();
    let context_local = v8::Local::new(scope, context);
    let global_obj: v8::Local<v8::Object> = context_local.global(scope);

    let gt_str = v8::String::new_external_onebyte_static(scope, b"globalThis")?;
    let gt_ns: v8::Local<v8::Object> = global_obj.get(scope, gt_str.into())?.try_into().ok()?;

    let func_name_str = v8::String::new(scope, func_name)?;
    let global_fn = gt_ns.get(scope, func_name_str.into())?;
    v8::Local::<v8::Function>::try_from(global_fn)
        .ok()
        .map(|x| v8::Global::new(scope, x))
}

fn set_global_func(
    js_runtime: &mut deno_core::JsRuntime,
    func_name: &'static str,
    source_code: &'static str,
) {
    let _ = js_runtime.global_realm().execute_script(
        js_runtime.v8_isolate(),
        func_name,
        deno_core::ModuleCode::from_static(source_code),
    );
}

fn execute_global_func(
    js_runtime: &mut deno_core::JsRuntime,
    func: v8::Global<v8::Function>,
    params: &Object,
) -> TegResult<Object> {
    let scope = &mut js_runtime.handle_scope();
    let func: v8::Local<v8::Function> = v8::Local::new(scope, func);
    let undefined = v8::undefined(scope);

    let arg_vals = [DenoUtils::wrap_value(&params, scope)];
    let ref mut try_catch = v8::TryCatch::new(scope);

    match func.call(try_catch, undefined.into(), &arg_vals) {
        Some(v) => DenoUtils::to_typed_value(v, try_catch).ok_or(ErrorCode::ScriptEvalFailed(
            format!("convert to object from {:?} failed", v),
        )),
        None => {
            return fmt_err!(
                ScriptEvalFailed,
                "eval javascript failed, error: {}",
                DenoUtils::try_catch_log(try_catch)
            );
        }
    }
}
