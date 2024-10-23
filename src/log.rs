use crate::{
    // logging::{LogDuration, LogError, LogMessage, RequestRecord},
};
#[macro_export]
macro_rules! _prompt_log {
    {
        logger: $logger:expr,
        is_access: $access:expr,
        condition: $cond:expr,
        prompt: [$($p:tt)*],
        standard: {$($std:tt)*}$(,)?
    } => {
        $crate::_prompt_log!{
            logger: $logger,
            is_access: $access,
            condition: $cond,
            prompt: [$($p)*],
            standard: {$($std)*},
            colored: {$($std)*},
        }
    };
    {
        logger: $logger:expr,
        is_access: $access:expr,
        condition: $cond:expr,
        prompt: [$($p:tt)*],
        standard: {
            formats: [$($std_fmt:tt)*],
            args: [$($std_args:expr),*$(,)?]$(,)?
        },
        colored: {
            formats: [$($col_fmt:tt)*],
            args: @$(,)?
        }$(,)?
    } => {
        $crate::_prompt_log!{
            logger: $logger,
            is_access: $access,
            condition: $cond,
            prompt: [$($p)*],
            standard: {
                formats: [$($std_fmt)*],
                args: [$($std_args),*],
            },
            colored: {
                formats: [$($col_fmt)*],
                args: [$($std_args),*],
            },
        }
    };
    {
        logger: $logger:expr,
        is_access: $access:expr,
        condition: $cond:expr,
        prompt: [$now:expr, $precise_time:expr, $pid:expr, $lvl:expr, $tag:expr$(,)?],
        standard: {
            formats: [$($std_fmt:tt)*],
            args: [$($std_args:expr),*$(,)?]$(,)?
        },
        colored: {
            formats: [$($col_fmt:tt)*],
            args: [$($col_args:expr),*$(,)?]$(,)?
        }$(,)?
    } => {
        if $cond {
            $crate::_prompt_log!(@bind [$logger, concat!("{} \x1b[2m{} \x1b[;2;1m{} {} \x1b[0;1m{}\x1b[m\t", $($col_fmt)*)] [$now, $precise_time, $pid, $lvl.as_str($access, true), $tag] $($col_args),*)
        } else {
            $crate::_prompt_log!(@bind [$logger, concat!("{} {} {} {} {}\t", $($std_fmt)*)] [$now, $precise_time, $pid, $lvl.as_str($access, false), $tag] $($std_args),*)
        }
    };
    (@bind [$logger:expr, $fmt:expr] [$($bindings:expr),*] $arg:expr $(, $args:expr)*) => {{
        let binding = &$arg;
        $crate::_prompt_log!(@bind [$logger, $fmt] [$($bindings),* , binding] $($args),*)
    }};
    (@bind [$logger:expr, $fmt:expr] [$($bindings:expr),*]) => {
        $logger(format_args!($fmt, $($bindings),*))
    };
}

#[macro_export]
macro_rules! _log_enabled {
    ($logger:expr, $lvl:expr) => {{
        let logger = $logger.borrow_mut();
        if !logger.enabled($crate::logging::Metadata {
            level: $lvl,
            target: module_path!(),
        }) {
            return;
        }
        logger
    }};
}

#[macro_export]
macro_rules! _log {
    ($lvl:expr, $format:expr $(, $args:expr)*) => {{
        $crate::logging::LOGGER.with(|logger| {
            let mut logger = $crate::_log_enabled!(logger, $lvl);
            let (pid, tag, inner) = logger.split();
            let (now, precise_time) = $crate::logging::now();

            $crate::_prompt_log!{
                logger: |args| inner.log(args),
                is_access: false,
                condition: inner.colored,
                prompt: [now, precise_time, pid, $lvl, tag],
                standard: {
                    formats: [$format, '\n'],
                    args: [$($args),*]
                }
            };
        })
    }};
}
#[macro_export]
macro_rules! error {
    ($format:expr $(, $args:expr)* $(,)?) => {
        $crate::_log!($crate::logging::LogLevel::Error, $format $(, $args)*)
    };
}