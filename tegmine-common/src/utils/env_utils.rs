use std::env;

use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumIter, EnumString};

use crate::prelude::*;

pub struct EnvUtils;

impl EnvUtils {
    pub fn is_environment_variable(test: &str) -> bool {
        for c in SystemParameters::iter() {
            if c.as_ref().eq(test) {
                return true;
            }
        }

        env::vars().find(|(k, _)| k.eq(test)).is_some()
    }

    pub fn get_system_parameters_value(
        sys_param: &str,
        task_id: Option<&InlineStr>,
    ) -> Option<InlineStr> {
        if SystemParameters::WfTaskId.as_ref().eq(sys_param) {
            if let Some(task_id) = task_id {
                Some(task_id.clone())
            } else {
                None
            }
        } else {
            if let Ok(v) = env::var(sys_param) {
                Some(v.into())
            } else {
                None
            }
        }
    }
}

#[derive(Clone, Copy, EnumString, EnumIter, AsRefStr)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
enum SystemParameters {
    WfTaskId,
    SfEnv,
    SfStack,
}
