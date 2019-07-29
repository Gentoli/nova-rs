use crate::core::reactor::{ReactorDatagram, ReactorFuture, ReactorFutureData};
use crossbeam::channel::{bounded, unbounded, Receiver, Sender};
use futures::task::Waker;
use std::sync::Arc;
use std::thread;

/// Single thread reactor type. Uses a single sacrificial thread to process work.
///
/// Designed to be used to turn an otherwise synchronous api into an async api through having a sacrificial thread do
/// the work. Construct with [`from_action`](#method.from_action). Is a thin layer around the internal reactor. Is
/// trivially clonable.
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
    /// Construct a reactor from a function that processes every input into an output.
    ///
    /// # Example
    ///
    /// ```edition2018
    /// # use nova_rs::core::reactor::SingleThreadReactor;
    /// // Reactor will double all inputs given to it.
    /// // This operation is generally non-cpu intensive
    /// // and otherwise "blocking".
    /// let reactor: SingleThreadReactor<i32, i32> = SingleThreadReactor::from_action(|x| x * 2);
    /// ```
    pub fn from_action<A>(f: A) -> Self
    where
        A: (Fn(S) -> R) + Send + 'static,
    {
        let (send, recv) = unbounded();
        let reactor = Arc::new(SingleThreadedReactorImpl { receiver: recv });
        {
            let reactor = reactor.clone();
            thread::spawn(move || reactor.run(f));
        }
        SingleThreadReactor { sender: send, reactor }
    }

    /// Send an input to the reactor for processing.
    ///
    /// # Example
    ///
    /// ```edition2018
    /// # #![feature(async_await)]
    /// # use futures::executor::block_on;
    /// # use nova_rs::core::reactor::SingleThreadReactor;
    /// # block_on(
    /// # async {
    /// let reactor = SingleThreadReactor::from_action(|x| x * 2);
    /// let answer = reactor.send_async(3).await;
    /// assert_eq!(answer, 6);
    /// # }
    /// # )
    /// ```
    pub fn send_async(&self, data: S) -> ReactorFuture<S, R> {
        ReactorFuture {
            data: ReactorFutureData::Unsent(data, self.clone()),
        }
    }

    pub(in crate::core::reactor) fn send(&self, data: S, waker: Waker) -> Receiver<R> {
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

/// Internal reactor. Contains only the receiver to receive new messages.
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
    /// Runs loop that runs the loop until the channel is hung up.
    fn run<A>(&self, action: A)
    where
        A: Fn(S) -> R + Send + 'static,
    {
        loop {
            match self.receiver.recv() {
                Err(_) => break,
                Ok(datagram) => {
                    let result = action(datagram.data);
                    let _ = datagram.sender.send(result);
                    datagram.waker.wake();
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::core::reactor::SingleThreadReactor;
    use futures::executor::LocalPool;
    use futures::task::LocalSpawnExt;

    #[test]
    fn remote_doubler() {
        let mut pool = LocalPool::new();
        let mut spawner = pool.spawner();

        let mut spawner2 = spawner.clone();

        spawner
            .spawn_local(async move {
                let reactor: SingleThreadReactor<i32, i32> = SingleThreadReactor::from_action(|x| x * 2);

                let mut array: Vec<_> = (0..100)
                    .map(|v| reactor.send_async(v))
                    .map(|f| spawner2.spawn_local_with_handle(f).expect("couldn't spawn future"))
                    .collect();

                for (i, f) in array.drain(0..).enumerate() {
                    assert_eq!(f.await, (i * 2) as i32);
                }
            })
            .expect("Spawn error");

        pool.run();
    }
}
