use std::{
    collections::{HashMap, hash_map::Entry},
    convert::Infallible,
    fs::File,
    sync::{Arc, Mutex, Weak},
};

use anyhow::Context;
use ropey::{Rope, RopeSlice};
use tokio::{
    select,
    sync::{broadcast, mpsc, oneshot},
    task::JoinHandle,
};

#[derive(Default)]
struct PluginSystem(Arc<Mutex<PluginSystemInner>>);

#[derive(Default)]
struct PluginSystemInner {
    plugins: Vec<Plugin>,
    plugin_by_call_lookup: HashMap<String, usize>,
    all_broadcast_recvs: HashMap<Broadcast, broadcast::Receiver<Value>>,
    pub all_command_senders: HashMap<Call, mpsc::Sender<(Value, oneshot::Sender<Value>)>>,
}

impl PluginSystem {
    /// This spawns an isolated plugin. It's I/O must be registered separately afterwards.
    ///
    /// For now, a single call is registered per plugin by default for testing.
    fn spawn<F, Fut>(&mut self, run: F)
    where
        F: FnOnce(Arc<PluginContext>) -> Fut,
        Fut: Future<Output = Infallible> + Send + 'static,
    {
        let plugin_system_weak = Arc::downgrade(&self.0);

        let mut inner = self.0.lock().unwrap();

        let (request_unlock_mutex_send, request_unlock_mutex_recv) = mpsc::channel(2);

        let plugin_context = Arc::new(PluginContext {
            locked: tokio::sync::Mutex::new(PluginContextLocked {
                request_unlock_mutex: request_unlock_mutex_recv,
                broadcast_senders: HashMap::default(),
                call_receivers: HashMap::default(),
            }),
            plugin_system: plugin_system_weak,
        });

        let task = {
            let plugin_context = plugin_context.clone();
            tokio::spawn(run(plugin_context))
        };

        let new_plugin = Plugin {
            task,
            context: plugin_context,
            request_context: request_unlock_mutex_send,
        };

        inner.plugins.push(new_plugin);
    }

    async fn call<'a>(&'a mut self, cmd: &str, args: Value) -> anyhow::Result<Value> {
        let inner = self.0.lock().unwrap();
        let sender = inner
            .all_command_senders
            .iter()
            .find(|(call, _)| &*call.0 == cmd)
            .context("command not found")?
            .1;

        let (returns_send, returns_recv) = oneshot::channel();

        sender.send((args, returns_send)).await.unwrap();

        drop(inner);
        let returns = returns_recv.await.expect("no errors");

        Ok(returns)
    }
}

struct PluginContext {
    locked: tokio::sync::Mutex<PluginContextLocked>,
    plugin_system: Weak<Mutex<PluginSystemInner>>,
}

struct PluginContextLocked {
    request_unlock_mutex: mpsc::Receiver<RequestUnlockMutex>,
    broadcast_senders: HashMap<Broadcast, broadcast::Sender<Value>>,
    call_receivers: HashMap<Call, mpsc::Receiver<(Value, oneshot::Sender<Value>)>>,
}

impl PluginContext {
    async fn send_broadcast(&self, broadcast: &Broadcast, args: Value) {
        self.locked
            .lock()
            .await
            .broadcast_senders
            .get(broadcast)
            .unwrap()
            .send(args)
            .unwrap();
    }

    async fn handle_call(&self, call: &Call) -> (Value, impl FnOnce(Value)) {
        let mut locked = self.locked.lock().await;

        // Replace with stream
        let calls = loop {
            let PluginContextLocked {
                request_unlock_mutex,
                call_receivers,
                ..
            } = &mut *locked;

            let y = select! {
                x = call_receivers.get_mut(call).unwrap().recv() => break x,
                y = request_unlock_mutex.recv() => y,
            };

            drop(locked);
            let MayLockMutexAgain = y.unwrap().0.try_recv().unwrap();
            locked = self.locked.lock().await;
        };

        let (value, finish) = calls.expect("channels are not dropped for now");

        (value, |returns| {
            finish
                .send(returns)
                .expect("oneshot channel is never dropped")
        })
    }

    async fn register_broadcast(&self, name: &str) -> Broadcast {
        let mut locked = self.locked.lock().await;

        let plugin_system = self
            .plugin_system
            .upgrade()
            .expect("plugin system never dropped");

        let mut plugin_system = plugin_system.lock().unwrap();

        let (broadcast_send, broadcast_recv) = broadcast::channel(16);

        let broadcast = Broadcast(Arc::from(name));
        locked
            .broadcast_senders
            .insert(broadcast.clone(), broadcast_send);

        match plugin_system.all_broadcast_recvs.entry(broadcast.clone()) {
            Entry::Occupied(_) => panic!("duplicate broadcast"),
            Entry::Vacant(vacant_entry) => vacant_entry.insert(broadcast_recv),
        };

        broadcast
    }

    async fn register_call(&self, name: &str) -> Call {
        let mut locked = self.locked.lock().await;

        let plugin_system = self
            .plugin_system
            .upgrade()
            .expect("plugin system never dropped");

        let mut plugin_system = plugin_system.lock().unwrap();

        let (call_send, call_recv) = mpsc::channel(16);

        let call = Call(Arc::from(name));
        locked.call_receivers.insert(call.clone(), call_recv);

        match plugin_system.all_command_senders.entry(call.clone()) {
            Entry::Occupied(_) => panic!("duplicate call"),
            Entry::Vacant(vacant_entry) => vacant_entry.insert(call_send),
        };

        call
    }
}

struct BufferStore {
    buf: Rope,

    buffer_new: Call,
    on_buffer_new: Broadcast,
}

impl BufferStore {
    pub async fn run(&mut self, context: Arc<PluginContext>) -> Infallible {
        loop {
            let (Value::Str(filepath), finish) = context.handle_call(&self.buffer_new).await else {
                panic!("invalid args")
            };

            self.buf = Rope::from_str(&filepath);
            finish(Value::Unit);

            context
                .send_broadcast(&self.on_buffer_new, Value::Str(filepath))
                .await;
        }
    }

    /// Entry point for this plugin
    async fn plugin_entry_point(context: Arc<PluginContext>) -> Infallible {
        let mut buffer_store = Self {
            buf: Rope::from_str("scratch buffer"),

            // Register plugin I/O
            // TODO: Check if such a delayed registration causes issues. These changes may not be
            // visible after calling `system.register(BufferStore::run_plugin)`
            buffer_new: context.register_call("buffer.new").await,
            on_buffer_new: context.register_broadcast("on.buffer.new").await,
        };

        buffer_store.run(context).await
    }
}

struct MayLockMutexAgain;
struct RequestUnlockMutex(oneshot::Receiver<MayLockMutexAgain>);

struct Plugin {
    task: JoinHandle<Infallible>,
    /// Required for when the plugin registers new or removes old commands. In those cases request
    /// exclusive access to the lockable context via `request_context`.
    context: Arc<PluginContext>,
    request_context: mpsc::Sender<RequestUnlockMutex>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Call(Arc<str>);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Broadcast(Arc<str>);

#[derive(Debug)]
enum ValueRef<'a> {
    Unit,
    U32(u32),
    U64(u64),
    Str(&'a str),
    Rope(RopeSlice<'a>),
    List(&'a [Value]),
}

#[derive(Debug, Clone)]
enum Value {
    Unit,
    U32(u32),
    U64(u64),
    Str(String),
    Rope(Rope),
    List(Vec<Value>),
}

// Two types of RPCs:
// Call: Multi Producer, Single Consumer. Consumer can return value to original producer.
// Broadcast: Single Producer, Multi Consumer. Producer does not see anything after sending.

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{BufferStore, PluginSystem, Value};

    #[tokio::test(flavor = "multi_thread")]
    async fn testing() {
        let mut plugin_system = PluginSystem::default();
        plugin_system.spawn(BufferStore::plugin_entry_point);

        tokio::time::sleep(Duration::from_secs(1)).await;

        plugin_system
            .0
            .lock()
            .unwrap()
            .all_broadcast_recvs
            .iter()
            .map(|x| (x.0.clone(), x.1.resubscribe()))
            .for_each(|(broadcast, mut receiver)| {
                println!("spawning broadcast listener");
                tokio::spawn(async move {
                    while let Ok(val) = receiver.recv().await {
                        println!("{broadcast:?}: {val:?}");
                    }
                });
            });

        loop {
            let x = plugin_system
                .call("buffer.new", Value::Str("hello.txt".to_owned()))
                .await;

            if x.is_ok() {
                println!("Success");
                break;
            }
        }

        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}
