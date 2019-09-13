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

/// Simple async function call wrapper.
///
/// This macro has three primary purposes:
/// - Remove the boilerplate from spawning an async call on an executor.
/// - Create an async call stack to enable easier debugging.
/// - Make all asynchronous calls use the same syntax for ease of use.
///
/// The syntax for the macro is as follows:
///
/// ```no_compile
/// async_invoke!(<mode>: (<ctx>,) <function>, (args: <args,>,) (executor: <executor>,) (stack: <call_stack>,) (handler: <handler>));
/// ```
///
/// All functions called by this macro must take a [`Context`] as the first argument. The name of the argument does not
/// matter. They must also all be `async`.
///
/// # Arguments:
/// - `mode` is the type of invocation. This is one of the following identifiers: **REQUIRED**
///   - Async Context:
///     - `exec` invokes the function on the provided executor using [`spawn_with_handle`].
///     - `inline` invokes the function directly.
///   - Sync Context:
///     - `from-sync` invokes the function on the provided executor using [`spawn_with_handle`]. It is the start of the
///       async call stack.
///     - `primary` invokes the function on the provided executor using [`run`]. It is the future that keeps the
///       executor running and will block until the future returns.
/// - `<ctx>` is the [`Context`] of the current async function. If you don't have a context, you are either in a sync
///   function or need to add a `mut ctx: Context` as the first argument of your function. **REQUIRED in an async
///   context**.
/// - `<function>` is the name of the function to call. This is a proper expression, so anything that can be called with
///   `()` works here, including superfish. **REQUIRED**
/// - `<args,>` is your args separated by commas. These are full expressions and will just be passed right through. Omit
///   if you have no arguments.
/// - `<executor>` is the executor to use. **REQUIRED in a sync context**. If omitted in an sync context, will use the
///   provided `ctx`'s executor instead.
/// - `<stack>` is the stack to use. **REQUIRED in a sync context**. If omitted in an sync context, will use the
///   provided `ctx`'s stack instead.
/// - `<handler>` is the error handler to use. The error handler is a function that will be passed to `map_err`. This
///   result is then passed to the try operator `?`. This is never required. If not provided it will use a simple
///   `unwrap`.
///
/// # Examples
///
/// These examples assume the following function exists:
///
/// ```edition2018
/// # #![feature(async_await)]
/// # use nova_rs::async_utils::Context;
/// # use nova_rs::async_invoke;
/// # use futures::executor::ThreadPoolBuilder;
/// async fn doubler(ctx: Context, v: i32) -> i32 {
///     v * 2
/// }
///
/// # let mut tp = ThreadPoolBuilder::new().create().unwrap();
/// # let res = async_invoke!(primary: doubler, executor: tp, args: 2);
/// # assert_eq!(res, 4);
/// ```
///
/// A normal async -> async call
///
/// ```edition2018
/// ```
///
/// [`Context`]: async_utils::Context
/// [`run`]: https://rust-lang-nursery.github.io/futures-api-docs/0.3.0-alpha.17/futures/executor/struct.ThreadPool.html#method.run
/// [`spawn_with_handle`]: https://rust-lang-nursery.github.io/futures-api-docs/0.3.0-alpha.17/futures/task/trait.SpawnExt.html#method.spawn_with_handle
#[macro_export]
macro_rules! async_invoke {
    // Invoke on the executor
    (exec: $ctx:expr, $func:expr $(, executor: $executor:expr)? $(, stack: $call_stack:expr)? $(, handler: $handler:expr)? $(, args: $($args:expr),+)? ) => {{
        use futures::task::SpawnExt;
        let new_executor = $crate::async_executor!($ctx $(, $executor)?).clone();
        let stack = $crate::async_call_stack!($ctx $(, $call_stack)?).add_stack_frame(file!(), line!(), column!());
        let new_context = $crate::async_utils::Context {
            executor: new_executor,
            call_stack: stack,
        };
        $crate::async_handler!($crate::async_executor!($ctx $(, $executor)?).spawn_with_handle($func(new_context, $($($args),+)?)) $(, $handler)?)
    }};
    // Invoke without calling off to the executor
    (inline: $ctx:expr, $func:expr $(, executor: $executor:expr)? $(, stack: $call_stack:expr)? $(, args: $($args:expr),+)? ) => {{
        use futures::task::SpawnExt;
        let new_executor = $crate::async_executor!($ctx $(, $executor)?).clone();
        let stack = $crate::async_call_stack!($ctx $(, $call_stack)?).add_stack_frame(file!(), line!(), column!());
        let new_context = $crate::async_utils::Context {
            executor: new_executor,
            call_stack: stack,
        };
        $func(new_context, $($($args),+)?)
    }};
    // Invoke on the executor from synchronous code (i.e. the start of a callstack)
    (from-sync: $func:expr, executor: $executor:expr $(, handler: $handler:expr)? $(, args: $($args:expr),+)?) => {{
        use futures::task::SpawnExt;
        let stack = $crate::async_utils::StackFrame::new(file!(), line!(), column!());
        let new_executor = $crate::async_executor!(x, $executor).clone();
        let new_context = $crate::async_utils::Context {
            executor: new_executor,
            call_stack: stack,
        };
        $crate::async_handler!($crate::async_executor!(x, $executor).spawn_with_handle($func(new_context, $($($args),+)?)) $(, $handler)?)
    }};
    // Invoke on the executor using `run` instead of `spawn_with_handle`
    (primary: $func:expr, executor: $executor:expr $(, handler: $handler:expr)? $(, args: $($args:expr),+)?) => {{
        use futures::task::SpawnExt;
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

    async fn async_sub_fn(mut ctx: Context, v: i32) -> i32 {
        println!("{:?}", ctx.call_stack);
        assert_eq!(v, 2);
        3
    }

    async fn async_fn(mut ctx: Context) {
        let f = async_invoke!(inline: ctx, async_sub_fn, args: 2);
        let v: i32 = f.await;
        assert_eq!(v, 3);
    }

    #[test]
    fn async_invoke() {
        let mut exec = ThreadPoolBuilder::new().create().unwrap();
        async_invoke!(primary: async_fn, executor: exec);
    }
}
