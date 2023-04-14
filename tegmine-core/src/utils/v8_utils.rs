use tegmine_common::prelude::*;

pub struct V8Utils;

impl V8Utils {
    // pub fn set_up_v8_globally() {
    //     let flags = concat!(" --turbo-fast-api-calls", " --single-threaded",);
    //     v8::V8::set_flags_from_string(flags);

    //     let platform = v8::new_single_threaded_default_platform(false).make_shared();
    //     v8::V8::initialize_platform(platform);
    //     v8::V8::initialize();
    // }

    pub fn set_up_v8_globally() {
        // v8::V8::set_flags_from_string(
        //     "--no_freeze_flags_after_init --expose_gc --harmony-import-assertions
        // --harmony-shadow-realm --allow_natives_syntax --turbo_fast_api_calls",   );
        v8::V8::initialize_platform(v8::new_default_platform(10, false).make_shared());
        v8::V8::initialize();
    }

    // wrap_value() will convert a TypedValue into v8::Value.
    pub fn wrap_value<'s, 'i, 'p>(
        typed_val: &'p Object,
        scope: &'i mut v8::HandleScope<'s, ()>,
    ) -> v8::Local<'s, v8::Value>
    where
        's: 'i,
    {
        match typed_val {
            Object::Int(value) => {
                let v8_number = v8::Integer::new(scope, *value);
                v8::Local::<v8::Value>::from(v8_number)
            }
            Object::Long(value) => {
                let ctx = v8::Context::new(scope);
                let context_scope = &mut v8::ContextScope::new(scope, ctx);

                let v8_i64 = v8::BigInt::new_from_i64(context_scope, *value);
                v8::Local::<v8::Value>::from(v8_i64)
            }
            // Object::Float(value) => {
            //     let v8_number = v8::Number::new(scope, (*value) as f32);
            //     v8::Local::<v8::Value>::from(v8_number)
            // }
            // Object::Double(value) => {
            //     let v8_number = v8::Number::new(scope, *value);
            //     v8::Local::<v8::Value>::from(v8_number)
            // }
            Object::Boolean(value) => {
                let v8_bool = v8::Boolean::new(scope, *value);
                v8::Local::<v8::Value>::from(v8_bool)
            }
            Object::String(value) => {
                let v8_str = v8::String::new(scope, value.as_str()).unwrap();
                v8::Local::<v8::Value>::from(v8_str)
            }
            Object::Map(value) => {
                let ctx = v8::Context::new(scope);
                let context_scope = &mut v8::ContextScope::new(scope, ctx);

                let v8_obj = if value.is_empty() {
                    v8::Object::new(context_scope)
                } else {
                    let v8_obj = v8::Object::new(context_scope);

                    value.iter().for_each(|(key, value)| {
                        let key = v8::String::new(context_scope, key.as_str()).unwrap();

                        let value = Self::wrap_value(value, context_scope);
                        v8_obj.set(context_scope, key.into(), value);
                    });
                    v8_obj
                };

                v8::Local::<v8::Value>::from(v8_obj)
            }
            Object::List(_) => {
                let val = serde_json::to_string(&typed_val.to_json()).unwrap();
                let val = v8::String::new(scope, &val).unwrap();

                let ctx = v8::Context::new(scope);
                let context_scope = &mut v8::ContextScope::new(scope, ctx);

                match v8::json::parse(context_scope, v8::Local::<v8::String>::from(val)) {
                    Some(local) => local,
                    None => v8::Local::from(v8::undefined(context_scope)),
                }
            }
            Object::Null => {
                let v8_null = v8::null(scope);
                v8::Local::<v8::Value>::from(v8_null)
            } // TODO: Invalid
        }
    }

    // to_typed_value() will convert v8::Value into TypedValue.
    pub fn to_typed_value<'s>(
        local: v8::Local<v8::Value>,
        handle_scope: &'s mut v8::HandleScope,
    ) -> Option<Object> {
        if local.is_undefined() {
            return Some(Object::Null); // TODO: use invalid instead
        }
        if local.is_int32() {
            return local.int32_value(handle_scope).map(|val| Object::Int(val));
        }
        if local.is_big_int() {
            return local
                .to_big_int(handle_scope)
                .filter(|val| {
                    let (_, ok) = val.i64_value();
                    ok
                })
                .map(|val| {
                    let (v, _) = val.i64_value();
                    Object::Long(v)
                });
        }
        if local.is_number() {
            // return local
            //     .number_value(handle_scope)
            //     .map(|val| Object::Double(val));
            unimplemented!("not support number")
        }
        if local.is_boolean() {
            return Some(Object::Boolean(local.is_true()));
        }
        if local.is_string() {
            return local
                .to_string(handle_scope)
                .map(|val| Object::String(val.to_rust_string_lossy(handle_scope).into()));
        }
        if local.is_object() {
            let args = v8::GetPropertyNamesArgsBuilder::default().build();
            return local.to_object(handle_scope).and_then(|obj| {
                obj.get_own_property_names(handle_scope, args).map(|names| {
                    let mut map = HashMap::default();
                    let arr = &*names;
                    for index in 0..arr.length() {
                        arr.get_index(handle_scope, index).iter().for_each(|key| {
                            let value = obj.get(handle_scope, key.clone()).unwrap();
                            let v = Self::to_typed_value(value, handle_scope).unwrap();
                            map.insert(key.to_rust_string_lossy(handle_scope).into(), v);
                        })
                    }
                    Object::Map(map)
                })
            });
        }
        if local.is_array() {
            return local.to_object(handle_scope).map(|obj| {
                let mut arr = vec![];
                let mut index = 0;
                loop {
                    let has_index_opt = obj.has_index(handle_scope, index);
                    if has_index_opt.is_some() && !has_index_opt.unwrap() {
                        break;
                    }
                    if has_index_opt.is_none() {
                        break;
                    }

                    let value_opts = obj.get_index(handle_scope, index);
                    if value_opts.is_none() {
                        break;
                    }

                    let val = value_opts.unwrap();
                    arr.push(Self::to_typed_value(val, handle_scope).unwrap_or(Object::Null)); // TODO: use invalid instead of null
                    index = index + 1;
                }

                Object::List(arr)
            });
        }
        if local.is_null() {
            return Some(Object::Null);
        }

        // Some(TypedValue::Invalid)
        unimplemented!()
    }

    pub fn try_catch_log(try_catch: &mut v8::TryCatch<v8::HandleScope>) -> String {
        let exception = try_catch.exception().unwrap();
        let exception_string = exception
            .to_string(try_catch)
            .unwrap()
            .to_rust_string_lossy(try_catch);
        // error!("{}", exception_string);
        exception_string
    }
}
