use crate::core::reactor::{ReactorDatagram, ReactorFuture, ReactorFutureData};
use crossbeam::channel::{bounded, unbounded, Receiver, Sender};
use futures::task::Waker;
use std::sync::Arc;
use std::thread;

/// Single threaded reactor type. Designed to be used to turn an otherwise synchronous api
/// into an async api through having a sacrificial thread do the work. Construct with
/// [`from_action`](#method.from_action)
pub struct SingleThreadReactor<S, R>
where
    S: Send + 'static,
    R: Send + 'static,
{
    sender: Sender<ReactorDatagram<S, R>>,
    reactor: Arc<SingleThreadedReactorImpl<S, R>>,
}

impl<S, R> SingleThreadReactor<S, R>
where
    S: Send + 'static,
    R: Send + 'static,
{
    pub fn from_action<A: (Fn(S) -> R) + Send + 'static>(f: A) -> Self {
        let (send, recv) = unbounded();
        let reactor = Arc::new(SingleThreadedReactorImpl { receiver: recv });
        {
            let reactor = reactor.clone();
            thread::spawn(move || reactor.run(f));
        }
        SingleThreadReactor { sender: send, reactor }
    }

    pub fn send_async(&self, data: S) -> ReactorFuture<S, R> {
        ReactorFuture {
            data: ReactorFutureData::Unsent(data, self.clone()),
        }
    }

    pub(crate) fn send(&self, data: S, waker: Waker) -> Receiver<R> {
        let (result_send, result_recv) = bounded(1);
        let _ = self.sender.send((data, waker, result_send).into());

        result_recv
    }
}

impl<S, R> Clone for SingleThreadReactor<S, R>
where
    S: Send + 'static,
    R: Send + 'static,
{
    fn clone(&self) -> Self {
        SingleThreadReactor {
            sender: self.sender.clone(),
            reactor: self.reactor.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.sender = source.sender.clone();
        self.reactor = source.reactor.clone();
    }
}

struct SingleThreadedReactorImpl<S, R>
where
    S: Send + 'static,
    R: Send + 'static,
{
    receiver: Receiver<ReactorDatagram<S, R>>,
}

impl<S, R> SingleThreadedReactorImpl<S, R>
where
    S: Send + 'static,
    R: Send + 'static,
{
    fn run<A: Fn(S) -> R + Send + 'static>(&self, action: A) {
        loop {
            match self.receiver.recv() {
                Ok(datagram) => {
                    let result = action(datagram.data);
                    datagram.waker.wake();
                    let _ = datagram.sender.send(result);
                }
                Err(_) => break,
            }
        }
    }
}

unsafe impl<S, R> Send for SingleThreadedReactorImpl<S, R>
where
    S: Send,
    R: Send,
{
}
unsafe impl<S, R> Sync for SingleThreadedReactorImpl<S, R>
where
    S: Send,
    R: Send,
{
}
