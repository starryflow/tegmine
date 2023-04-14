use tegmine_common::prelude::*;

use crate::utils::V8Utils;

/// [`RuntimeEngine`] wraps an isolated instance of v8 engine which contains only one function's
/// runtime context. A [`RuntimeEngine`] can only be initialized in a single-thread. It cannot be
/// shared or transmitted between multi-threads. The lifecycle of [`RuntimeEngine`]:
/// - initialized
/// - running
/// - dropped
pub struct RuntimeEngine<'s, 'i> {
    context_scope: v8::ContextScope<'i, v8::HandleScope<'s>>,
    ctx: v8::Local<'s, v8::Context>,
    process_fn: Option<v8::Local<'s, v8::Function>>,
}

impl<'s, 'i> RuntimeEngine<'s, 'i>
where
    's: 'i,
{
    pub fn new(
        source_code: &str,
        fn_name: &str,
        isolated_scope: &'i mut v8::HandleScope<'s, ()>,
    ) -> Self {
        let ctx = v8::Context::new(isolated_scope);
        let mut scope = v8::ContextScope::new(isolated_scope, ctx);
        let code = v8::String::new(&mut scope, source_code).unwrap();

        let script = v8::Script::compile(&mut scope, code, None).unwrap();

        let mut self_ = Self {
            context_scope: scope,
            ctx,
            process_fn: None,
        };

        let _ = self_.execute_script(script);

        let process_str = v8::String::new(&mut self_.context_scope, fn_name);

        let fn_value = ctx
            .global(&mut self_.context_scope)
            .get(&mut self_.context_scope, process_str.unwrap().into())
            .unwrap();
        let fn_opt = v8::Local::<v8::Function>::try_from(fn_value);
        let process_fn = if fn_opt.is_ok() {
            Some(fn_opt.unwrap())
        } else {
            None
        };

        self_.process_fn = process_fn;
        self_
    }

    /// call with one argument
    pub fn call_one_arg(&mut self, val: &Object) -> TegResult<Object> {
        let scope = &mut v8::HandleScope::new(&mut self.context_scope);
        let ref mut try_catch = v8::TryCatch::new(scope);
        let global = self.ctx.global(try_catch).into();
        let arg_vals = [V8Utils::wrap_value(val, try_catch)];

        let process_fn = self.process_fn.as_ref().unwrap();

        match process_fn.call(try_catch, global, &arg_vals) {
            Some(v) => V8Utils::to_typed_value(v, try_catch).ok_or(ErrorCode::ScriptEvalFailed(
                format!("convert to object from {:?} failed", v),
            )),
            None => {
                fmt_err!(
                    ScriptEvalFailed,
                    "eval javascript failed, error: {}",
                    V8Utils::try_catch_log(try_catch)
                )
            }
        }
    }

    /// call with two arguments
    pub fn _call_two_args(&mut self, vals: (&Object, &Object)) -> TegResult<Object> {
        let scope = &mut v8::HandleScope::new(&mut self.context_scope);
        let ref mut try_catch = v8::TryCatch::new(scope);
        let global = self.ctx.global(try_catch).into();
        let process_fn = self.process_fn.as_mut().unwrap();
        let arg_vals = &[
            V8Utils::wrap_value(vals.0, try_catch),
            V8Utils::wrap_value(vals.1, try_catch),
        ];

        match process_fn.call(try_catch, global, arg_vals) {
            Some(v) => V8Utils::to_typed_value(v, try_catch).ok_or(ErrorCode::ScriptEvalFailed(
                format!("convert to object from {:?} failed", v),
            )),
            None => {
                fmt_err!(
                    ScriptEvalFailed,
                    "eval javascript failed, error: {}",
                    V8Utils::try_catch_log(try_catch)
                )
            }
        }
    }

    fn execute_script(&mut self, script: v8::Local<'s, v8::Script>) -> TegResult<()> {
        let handle_scope = &mut v8::HandleScope::new(&mut self.context_scope);
        let try_catch = &mut v8::TryCatch::new(handle_scope);

        if script.run(try_catch).is_none() {
            fmt_err!(
                ScriptEvalFailed,
                "execute javascript failed, error: {}",
                V8Utils::try_catch_log(try_catch)
            )
        } else {
            Ok(())
        }
    }
}
