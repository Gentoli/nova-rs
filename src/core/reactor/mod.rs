//! Event loop reactors to turn blocking operations into async operations.

use crossbeam::channel::{Receiver, Sender};
use futures::task::{Context, Waker};
use futures::{Future, Poll};
use std::mem;
use std::pin::Pin;

mod multi_thread;
mod single_thread;

pub use multi_thread::*;
pub use single_thread::*;

/// Current state of the reactor.
enum ReactorFutureData<S, R>
where
    S: Send + 'static,
    R: Send + 'static,
{
    Unsent(S, SingleThreadReactor<S, R>),
    Uninit,
    Sent(Receiver<R>),
    Finished,
}

/// Future representing a computation happening on a [`SingleThreadReactor`].
///
/// First time poll is called, sets up the computation, then will return pending until the answer arrives.
/// Currently only supports the [`SingleThreadReactor`].
/// This will be changed in the future.
pub struct ReactorFuture<S, R>
where
    S: Send + 'static,
    R: Send + 'static,
{
    data: ReactorFutureData<S, R>,
}

impl<S, R> Future for ReactorFuture<S, R>
where
    S: Send + 'static,
    R: Send + 'static,
{
    type Output = R;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let old_data = mem::replace(&mut self.data, ReactorFutureData::Uninit);
        let (new_data, result) = match old_data {
            ReactorFutureData::Unsent(data, reactor) => {
                let recv = reactor.send(data, cx.waker().clone());

                (ReactorFutureData::Sent(recv), Poll::Pending)
            }
            ReactorFutureData::Sent(receiver) => (
                ReactorFutureData::Finished,
                Poll::Ready(receiver.recv().expect("Expected receiver to have data")),
            ),
            _ => panic!("Incorrect state in reactor future. This is a bug."),
        };
        self.data = new_data;
        result
    }
}

impl<S, R> Unpin for ReactorFuture<S, R>
where
    S: Send + 'static,
    R: Send + 'static,
{
}

/// One message sent to the reactor. Contains the data, the waker to awake the waiting future,
/// and the sender to send the data back.
struct ReactorDatagram<S, R>
where
    S: Send + 'static,
    R: Send + 'static,
{
    pub data: S,
    pub waker: Waker,
    pub sender: Sender<R>,
}

impl<S, R> From<(S, Waker, Sender<R>)> for ReactorDatagram<S, R>
where
    S: Send + 'static,
    R: Send + 'static,
{
    fn from(tuple: (S, Waker, Sender<R>)) -> Self {
        Self {
            data: tuple.0,
            waker: tuple.1,
            sender: tuple.2,
        }
    }
}
