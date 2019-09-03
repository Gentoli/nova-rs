//! Utility structures for making writing async code easier.

use futures::executor::ThreadPool;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::sync::Arc;

pub struct Context {
    pub executor: ThreadPool,
    pub call_stack: Arc<StackFrame>,
}

pub struct StackFrame {
    file: &'static str,
    line: u32,
    column: u32,
    last: Option<Arc<StackFrame>>,
}

impl StackFrame {
    pub fn new(file: &'static str, line: u32, column: u32) -> Arc<StackFrame> {
        Arc::new(StackFrame {
            file,
            line,
            column,
            last: None,
        })
    }

    pub fn add_stack_frame(self: Arc<Self>, file: &'static str, line: u32, column: u32) -> Arc<StackFrame> {
        Arc::new(StackFrame {
            file,
            line,
            column,
            last: Some(Arc::clone(&self)),
        })
    }
}

impl Debug for StackFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(f, "{}:{}:{}", self.file, self.line, self.column)?;
        self.last.as_ref().map(|l| l.fmt(f)).unwrap_or(Ok(()))
    }
}

/// Helper function to allow a error handler to be used
#[doc(hidden)]
#[macro_export]
macro_rules! async_handler {
    ($rest:expr, $handler:expr) => {
        $rest.map_err($handler)?
    };
    ($rest:expr) => {
        $rest.unwrap()
    };
}

/// Helper function to allow a custom executor to be used
#[doc(hidden)]
#[macro_export]
macro_rules! async_executor {
    ($ctx:expr, $executor:expr) => {
        $executor
    };
    ($ctx:expr) => {
        $ctx.executor
    };
}

/// Helper function to allow a custom call stack to be used
#[doc(hidden)]
#[macro_export]
macro_rules! async_call_stack {
    ($ctx:expr, $stack:expr) => {
        $stack
    };
    ($ctx:expr) => {
        $ctx.call_stack
    };
}

#[macro_export]
macro_rules! async_invoke {
    (ctx: $ctx:expr, $func:expr $(, args: $($args:expr),+)? $(, executor: $executor:expr)? $(, stack: $call_stack:expr)? $(, handler: $handler:expr)?) => {{
        let new_executor = $crate::async_executor!($ctx $(, $executor)?).clone();
        let stack = $crate::async_call_stack!($ctx $(, $call_stack)?).add_stack_frame(file!(), line!(), column!());
        let new_context = $crate::async_utils::Context {
            executor: new_executor,
            call_stack: stack,
        };
        $crate::async_handler!($crate::async_executor!($ctx $(, $executor)?).spawn_with_handle($func(new_context, $($($args),+)?)) $(, $handler)?)
    }};
    (from-sync: $func:expr $(, args: $($args:expr),+)?, executor: $executor:expr $(, handler: $handler:expr)?) => {{
        let stack = $crate::async_utils::StackFrame::new(file!(), line!(), column!());
        let new_executor = $crate::async_executor!(x, $executor).clone();
        let new_context = $crate::async_utils::Context {
            executor: new_executor,
            call_stack: stack,
        };
        $crate::async_handler!($crate::async_executor!(x, $executor).spawn_with_handle($func(new_context, $($($args),+)?)) $(, $handler)?)
    }};
    (primary: $func:expr $(, args: $($args:expr),+)?, executor: $executor:expr $(, handler: $handler:expr)?) => {{
        let stack = $crate::async_utils::StackFrame::new(file!(), line!(), column!());
        let new_executor = $crate::async_executor!(x, $executor).clone();
        let new_context = $crate::async_utils::Context {
            executor: new_executor,
            call_stack: stack,
        };
        $crate::async_executor!(x, $executor).run($func(new_context, $($($args),+)?))
    }};
}

#[cfg(test)]
mod test {
    use crate::async_utils::Context;
    use futures::executor::ThreadPoolBuilder;
    use futures::task::SpawnExt;

    async fn async_sub_fn(mut ctx: Context, v: i32) -> i32 {
        println!("{:?}", ctx.call_stack);
        assert_eq!(v, 2);
        3
    }

    async fn async_fn(mut ctx: Context) {
        let f = async_invoke!(ctx: ctx, async_sub_fn, args: 2);
        let v: i32 = f.await;
        assert_eq!(v, 3);
    }

    #[test]
    fn async_invoke() {
        let mut exec = ThreadPoolBuilder::new().create().unwrap();
        async_invoke!(primary: async_fn, executor: exec);
    }
}
