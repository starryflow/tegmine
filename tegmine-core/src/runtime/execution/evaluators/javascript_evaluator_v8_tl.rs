use tegmine_common::prelude::*;

use super::Evaluator;
use crate::utils::V8Utils;

pub struct JavascriptEvaluatorV8Tl;

impl Evaluator for JavascriptEvaluatorV8Tl {
    fn evaluate(&self, expression: &InlineStr, input: &Object) -> TegResult<Object> {
        debug!("Javascript evaluator -- expression: {}", expression);
        // Evaluate the expression by using the Javascript evaluation engine.

        let fn_name = "process";
        let source_code = &format!("function {}($) {{ return {} }}", fn_name, expression);

        let eval_result = ISOLATE_BUFFER.with(|x| {
            let mut guard = x.borrow_mut();
            if guard.as_ref().is_none() {
                *guard = Some(new_isolate());
            }
            let isolate = guard.as_mut().expect("will not happen");

            CONTEXT_BUFFER.with(|y| {
                let mut guard_ = y.borrow_mut();
                if guard_.as_ref().is_none() {
                    *guard_ = Some(new_context(isolate));
                }
                let global_context = guard_.as_ref().expect("will not happen");

                let mut isolate_scope = v8::HandleScope::new(isolate);
                let context = v8::Local::new(&mut isolate_scope, global_context);
                let mut ctx_scope = v8::ContextScope::new(&mut isolate_scope, context);

                let process_str = v8::String::new(&mut ctx_scope, fn_name).expect("msg");
                error!("{:?}", process_str);
                if context
                    .global(&mut ctx_scope)
                    .get(&mut ctx_scope, process_str.into())
                    .unwrap()
                    .is_undefined()
                {
                    let code = v8::String::new(&mut ctx_scope, source_code).unwrap();
                    let script = v8::Script::compile(&mut ctx_scope, code, None).unwrap();
                    error!("script: {:?}", script);
                    // {
                    //     let handle_scope = &mut v8::HandleScope::new(&mut ctx_scope);
                    //     let try_catch = &mut v8::TryCatch::new(handle_scope);

                    //     script.run(try_catch).expect("msg");
                    // }
                    let fn_value = context
                        .global(&mut ctx_scope)
                        .get(&mut ctx_scope, process_str.into())
                        .unwrap();
                    error!("fn_value 1: {:?}", fn_value);
                }

                let fn_value = context
                    .global(&mut ctx_scope)
                    .get(&mut ctx_scope, process_str.into())
                    .unwrap();
                error!("fn_value 2: {:?}", fn_value);
                error!("{:?}", fn_value.is_undefined());

                let fn_value = context
                    .global(&mut ctx_scope)
                    .get(&mut ctx_scope, process_str.into())
                    .unwrap();
                error!("fn_value 3: {:?}", fn_value);
                error!("{:?}", fn_value.is_undefined());
                let process_fn = v8::Local::<v8::Function>::try_from(fn_value).expect("msg");

                {
                    let scope = &mut v8::HandleScope::new(&mut ctx_scope);
                    let mut try_catch = v8::TryCatch::new(scope);
                    let global = context.global(&mut try_catch).into();
                    let arg_vals = [V8Utils::wrap_value(input, &mut try_catch)];

                    match process_fn.call(&mut try_catch, global, &arg_vals) {
                        Some(v) => V8Utils::to_typed_value(v, &mut try_catch).ok_or(
                            ErrorCode::ScriptEvalFailed(format!(
                                "convert to object from {:?} failed",
                                v
                            )),
                        ),
                        None => {
                            fmt_err!(
                                ScriptEvalFailed,
                                "eval javascript failed, error: {}",
                                V8Utils::try_catch_log(&mut try_catch)
                            )
                        }
                    }
                }
            })
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

thread_local! {
  pub static ISOLATE_BUFFER: RefCell<Option<v8::OwnedIsolate>> = RefCell::new(None);
}

thread_local! {
  pub static CONTEXT_BUFFER: RefCell<Option<v8::Global<v8::Context>>> = RefCell::new(None);
}

pub fn new_isolate() -> v8::OwnedIsolate {
    let mut isolate = v8::Isolate::new(Default::default());
    isolate.set_microtasks_policy(v8::MicrotasksPolicy::Auto);
    isolate.low_memory_notification();
    isolate
}

pub fn new_context(isolate: &mut v8::OwnedIsolate) -> v8::Global<v8::Context> {
    let context_template: v8::Global<v8::ObjectTemplate> = {
        let scope = &mut v8::HandleScope::new(isolate);

        let object = v8::ObjectTemplate::new(scope);
        object.set(
            v8::String::new(scope, "some_global_resource_key")
                .unwrap()
                .into(),
            v8::String::new(scope, "some_global_resource")
                .unwrap()
                .into(),
        );
        v8::Global::new(scope, object)
    };

    {
        let scope = &mut v8::HandleScope::new(isolate);

        let template = v8::Local::new(scope, context_template);
        let ctx = v8::Context::new_from_template(scope, template);
        v8::Global::new(scope, ctx)
    }
}
