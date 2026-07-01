use crate::kinds::TaskKind;
use crate::task::FromInputs;
use crate::task::IntoRawKey;
use crate::task::TaskNode;
use crate::task::TypedJoinTask;
use crate::task::TypedJoinTasks;
use crate::task::TypedTask;
use crate::types::RawKey;
use crate::types::TaskIO;
use crate::types::TaskKey;
use crate::types::TaskProfile;
use petgraph::graph::Graph;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use rs_foundation::unsafe_type_wrapper::UnsafeTypeWrapper;
use slotmap::SlotMap;
use std::any::Any;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

pub struct TaskGraph {
    pub profiles: Arc<Mutex<HashMap<RawKey, TaskProfile>>>,
    pub io: Arc<Mutex<HashMap<RawKey, TaskIO>>>,
    pub graph: Graph<RawKey, ()>,
    pub tasks: Arc<Mutex<SlotMap<RawKey, Arc<dyn TaskNode>>>>,
    pub node_lookup: HashMap<RawKey, NodeIndex>,
    pub cache: Arc<Mutex<HashMap<RawKey, Arc<dyn Any + Send + Sync>>>>,
}

impl TaskGraph {
    pub fn new() -> TaskGraph {
        TaskGraph {
            profiles: Arc::new(Mutex::new(HashMap::new())),
            graph: Graph::new(),
            tasks: Arc::new(Mutex::new(SlotMap::with_key())),
            node_lookup: HashMap::new(),
            cache: Arc::new(Mutex::new(HashMap::new())),
            io: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn source<O, F>(&mut self, name: impl Into<String>, f: F) -> TaskKey<(), O>
    where
        O: Send + Sync + 'static + Debug,
        F: Fn() -> Result<O, String> + Send + Sync + 'static,
    {
        let task = TypedTask::<(), O, _> {
            name: name.into(),
            kind: TaskKind::Source,
            f: move |_unit: &()| f(),
            _marker: std::marker::PhantomData,
        };

        let raw = self.insert_task(task);
        TaskKey {
            raw,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn map<I, O, O2, F>(
        &mut self,
        name: impl Into<String>,
        input: TaskKey<I, O>,
        f: F,
    ) -> TaskKey<O, O2>
    where
        I: Send + Sync + 'static,
        O: Send + Sync + 'static + Debug,
        O2: Send + Sync + 'static + Debug,
        F: Fn(&O) -> Result<O2, String> + Send + Sync + 'static,
    {
        let task = TypedTask::<O, O2, _> {
            name: name.into(),
            kind: TaskKind::Map,
            f,
            _marker: std::marker::PhantomData,
        };

        let raw = self.insert_task(task);
        self.add_dependency(raw, input.raw);

        TaskKey {
            raw,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn sink<I, O, F>(&mut self, name: impl Into<String>, input: TaskKey<I, O>, f: F)
    where
        I: Send + Sync + 'static,
        O: Send + Sync + 'static + Debug,
        F: Fn(&O) -> Result<(), String> + Send + Sync + 'static,
    {
        let task = TypedTask::<O, (), _> {
            name: name.into(),
            kind: TaskKind::Sink,
            f,
            _marker: std::marker::PhantomData,
        };

        let raw = self.insert_task(task);
        self.add_dependency(raw, input.raw);
    }

    fn insert_task<T>(&mut self, task: T) -> RawKey
    where
        T: TaskNode + 'static,
    {
        let raw = self
            .tasks
            .lock()
            .unwrap()
            .insert(Arc::new(task) as Arc<dyn TaskNode>);
        let node = self.graph.add_node(raw);
        self.node_lookup.insert(raw, node);
        raw
    }

    pub fn remove_task(&mut self, key: RawKey) -> Vec<RawKey> {
        if !self.node_lookup.contains_key(&key) {
            return vec![];
        }

        let mut removed = vec![key];
        let mut stack = vec![key];
        let mut visited = std::collections::HashSet::new();
        visited.insert(key);

        while let Some(current_key) = stack.pop() {
            if let Some(&current_node) = self.node_lookup.get(&current_key) {
                for edge in self
                    .graph
                    .edges_directed(current_node, petgraph::Direction::Outgoing)
                {
                    let target_key = self.graph[edge.target()];
                    if visited.insert(target_key) {
                        stack.push(target_key);
                        removed.push(target_key);
                    }
                }
            }
        }

        let nodes_to_remove: Vec<NodeIndex> = self
            .graph
            .node_indices()
            .filter(|&idx| visited.contains(&self.graph[idx]))
            .collect();

        for node in nodes_to_remove.iter().rev() {
            self.graph.remove_node(*node);
        }

        for &k in &removed {
            self.tasks.lock().unwrap().remove(k);
            self.cache.lock().unwrap().remove(&k);
            self.profiles.lock().unwrap().remove(&k);
            self.io.lock().unwrap().remove(&k);
        }

        self.node_lookup.clear();
        for node_idx in self.graph.node_indices() {
            self.node_lookup.insert(self.graph[node_idx], node_idx);
        }

        removed
    }

    fn add_dependency(&mut self, task: RawKey, depends_on: RawKey) {
        let t = self.node_lookup[&task];
        let d = self.node_lookup[&depends_on];
        self.graph.add_edge(d, t, ());
    }

    pub fn execute(&mut self, threads: usize) -> Result<(), String> {
        let start_queue = std::time::Instant::now();
        let node_count = self.graph.node_count();

        let valid_indices: Vec<NodeIndex> = self.graph.node_indices().collect();
        let max_idx = valid_indices.iter().map(|n| n.index()).max().unwrap_or(0);
        let remaining: Vec<AtomicUsize> = (0..=max_idx)
            .map(|_| AtomicUsize::new(usize::MAX))
            .collect();
        let remaining = Arc::new(remaining);

        for &idx in &valid_indices {
            let indeg = self
                .graph
                .edges_directed(idx, petgraph::Direction::Incoming)
                .count();
            remaining[idx.index()].store(indeg, Ordering::SeqCst);
        }

        let queue = Arc::new(Mutex::new(VecDeque::<NodeIndex>::new()));
        let outputs: Arc<Mutex<HashMap<RawKey, Arc<dyn Any + Send + Sync>>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let aborted = Arc::new(AtomicBool::new(false));
        let first_error = Arc::new(Mutex::new(None));
        let completed = Arc::new(AtomicUsize::new(0));
        let cv = Arc::new(Condvar::new());

        for &idx in &valid_indices {
            if remaining[idx.index()].load(Ordering::SeqCst) == 0 {
                queue.lock().unwrap().push_back(idx);
            }
        }

        let mut handles = Vec::new();

        for _ in 0..threads {
            let graph = UnsafeTypeWrapper::from_mut_ref(&mut self.graph);
            let remaining = Arc::clone(&remaining);
            let tasks = Arc::clone(&self.tasks);
            let queue = Arc::clone(&queue);
            let outputs = Arc::clone(&outputs);
            let aborted = Arc::clone(&aborted);
            let first_error = Arc::clone(&first_error);
            let completed = Arc::clone(&completed);
            let cv = Arc::clone(&cv);
            let cached = Arc::clone(&self.cache);
            let profiles = Arc::clone(&self.profiles);
            let io = Arc::clone(&self.io);

            let handle = std::thread::spawn(move || {
                loop {
                    if aborted.load(Ordering::SeqCst) {
                        break;
                    }

                    let node = {
                        let mut q = queue.lock().unwrap();
                        loop {
                            if let Some(n) = q.pop_front() {
                                break n;
                            }
                            if aborted.load(Ordering::SeqCst)
                                || completed.load(Ordering::SeqCst) >= node_count
                            {
                                return;
                            }
                            q = cv.wait(q).unwrap();
                        }
                    };

                    let raw = graph[node];
                    let task: Arc<dyn TaskNode> = {
                        let guard = tasks.lock().unwrap();
                        Arc::clone(&guard[raw])
                    };

                    let mut inputs: Vec<Arc<dyn Any + Send + Sync>> = Vec::new();
                    {
                        let outs = outputs.lock().unwrap();
                        for edge in graph.edges_directed(node, petgraph::Direction::Incoming) {
                            let parent = edge.source();
                            if let Some(v) = outs.get(&graph[parent]) {
                                inputs.push(Arc::clone(v));
                            }
                        }
                    }
                    if inputs.is_empty() {
                        inputs.push(Arc::new(()) as Arc<dyn Any + Send + Sync>);
                    }
                    inputs.reverse();

                    if let Some(cached) = cached.lock().unwrap().get(&raw) {
                        outputs.lock().unwrap().insert(raw, Arc::clone(cached));
                    } else {
                        let queue_time = start_queue.elapsed();
                        let start_exec = std::time::Instant::now();

                        match task.run(&inputs) {
                            Ok(Some(out)) => {
                                let exec_time = start_exec.elapsed();
                                let thread_id = std::thread::current().id();
                                profiles.lock().unwrap().insert(
                                    raw,
                                    TaskProfile {
                                        queue_time,
                                        exec_time,
                                        thread_id,
                                    },
                                );

                                let input_strings = inputs
                                    .iter()
                                    .map(|v| task.format_input(v))
                                    .collect::<Vec<_>>();
                                let output_string = task.format_output(&out);
                                io.lock().unwrap().insert(
                                    raw,
                                    TaskIO {
                                        inputs: input_strings,
                                        output: Some(output_string),
                                    },
                                );

                                outputs.lock().unwrap().insert(raw, Arc::clone(&out));
                                cached.lock().unwrap().insert(raw, out);
                            }
                            Ok(None) => {}
                            Err(e) => {
                                aborted.store(true, Ordering::SeqCst);
                                let mut fe = first_error.lock().unwrap();
                                if fe.is_none() {
                                    *fe = Some(e);
                                }
                                cv.notify_all();
                                break;
                            }
                        }
                    }

                    completed.fetch_add(1, Ordering::SeqCst);

                    for edge in graph.edges_directed(node, petgraph::Direction::Outgoing) {
                        let child = edge.target().index();
                        let prev = remaining[child].fetch_sub(1, Ordering::SeqCst);
                        if prev == 1 {
                            queue.lock().unwrap().push_back(NodeIndex::new(child));
                        }
                    }

                    cv.notify_all();
                }
            });
            handles.push(handle);
        }

        for h in handles {
            let _ = h.join();
        }

        if let Some(err) = first_error.lock().unwrap().take() {
            Err(err)
        } else {
            Ok(())
        }
    }
}

impl TaskGraph {
    pub fn join_raw<T, O, F, const N: usize>(
        &mut self,
        name: impl Into<String>,
        inputs: [RawKey; N],
        f: F,
    ) -> RawKey
    where
        T: FromInputs + Send + Sync + 'static,
        O: Send + Sync + 'static,
        F: Fn(T) -> Result<O, String> + Send + Sync + 'static,
    {
        let task = TypedJoinTask::<T, O, F> {
            name: name.into(),
            f,
            _marker: std::marker::PhantomData,
        };

        let raw = self.insert_task(task);

        for inp in inputs {
            self.add_dependency(raw, inp);
        }

        raw
    }

    pub fn joins_raw<O, F>(
        &mut self,
        name: impl Into<String>,
        inputs: Vec<RawKey>,
        f: F,
    ) -> TaskKey<(), O>
    where
        O: Send + Sync + 'static,
        F: Fn(&[Arc<dyn Any + Send + Sync>]) -> Result<O, String> + Send + Sync + 'static,
    {
        let task = TypedJoinTasks::<O, F> {
            name: name.into(),
            f,
            _marker: std::marker::PhantomData,
        };

        let raw = self.insert_task(task);

        for inp in inputs {
            self.add_dependency(raw, inp);
        }

        TaskKey {
            raw,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn join<T, O, F, I>(&mut self, name: impl Into<String>, inputs: I, f: F) -> RawKey
    where
        T: FromInputs + Send + Sync + 'static,
        O: Send + Sync + 'static,
        F: Fn(T) -> Result<O, String> + Send + Sync + 'static,
        I: IntoIterator,
        I::Item: IntoRawKey,
    {
        let raw_inputs: Vec<RawKey> = inputs.into_iter().map(|k| k.into_raw()).collect();

        let task = TypedJoinTask::<T, O, F> {
            name: name.into(),
            f,
            _marker: std::marker::PhantomData,
        };

        let raw = self.insert_task(task);

        for inp in raw_inputs {
            self.add_dependency(raw, inp);
        }

        raw
    }

    pub fn joins<O, F, I, T, O2>(
        &mut self,
        name: impl Into<String>,
        inputs: T,
        f: F,
    ) -> TaskKey<(), O>
    where
        O: Send + Sync + 'static,
        F: Fn(&[Arc<dyn Any + Send + Sync>]) -> Result<O, String> + Send + Sync + 'static,
        T: IntoIterator<Item = TaskKey<I, O2>>,
        I: Send + Sync + 'static,
    {
        let raw_inputs: Vec<RawKey> = inputs.into_iter().map(|k| k.raw).collect();

        let task = TypedJoinTasks::<O, F> {
            name: name.into(),
            f,
            _marker: std::marker::PhantomData,
        };

        let raw = self.insert_task(task);

        for inp in raw_inputs {
            self.add_dependency(raw, inp);
        }

        TaskKey {
            raw,
            _marker: std::marker::PhantomData,
        }
    }
}

impl TaskGraph {
    pub fn to_dot_simple(&self) -> String {
        use petgraph::dot::{Config, Dot};

        let tasks = self.tasks.lock().unwrap();
        let graph = self.graph.map(
            |_node_idx, raw_key| {
                let task = &tasks[*raw_key];
                task.name().to_string()
            },
            |_, _| "".to_string(),
        );

        format!("{}", Dot::with_config(&graph, &[Config::EdgeNoLabel]))
    }

    pub fn to_dot(&self) -> String {
        let mut s = String::new();
        let tasks = self.tasks.lock().unwrap();

        s.push_str("digraph TaskGraph {\n");
        s.push_str("  rankdir=LR;\n");
        s.push_str("  node [shape=box, style=filled, fontname=\"Consolas\"];\n\n");

        let mut groups: std::collections::BTreeMap<TaskKind, Vec<petgraph::graph::NodeIndex>> =
            std::collections::BTreeMap::new();

        for node_idx in self.graph.node_indices() {
            let raw = self.graph[node_idx];
            let task = &tasks[raw];
            groups.entry(task.kind()).or_default().push(node_idx);
        }

        for (kind, nodes) in &groups {
            s.push_str(&format!("  subgraph cluster_{} {{\n", kind));
            s.push_str(&format!("    label=\"{}\";\n", kind));
            s.push_str("    style=dashed;\n");

            for node_idx in nodes {
                let raw = self.graph[*node_idx];
                let task = &tasks[raw];

                let mut label = if let Some(ti) = task.type_info() {
                    format!("{}\\n{}", task.name(), ti)
                } else {
                    task.name().to_string()
                };

                let color = match task.kind() {
                    TaskKind::Source => "#A5D6A7",
                    TaskKind::Map => "#90CAF9",
                    TaskKind::Join => "#FFCC80",
                    TaskKind::Sink => "#CE93D8",
                };

                label.push_str(&format!("\\n{:?}", node_idx));

                if let Some(io) = self.io.lock().unwrap().get(&raw) {
                    label.push_str(&format!(
                        "\\nin=[{}]\\nout={}",
                        io.inputs.join(","),
                        io.output.clone().unwrap_or_default(),
                    ));
                }

                if let Some(profile) = self.profiles.lock().unwrap().get(&raw) {
                    label.push_str(&format!(
                        "\\nexec={:.3}ms\\nqueue={:.3}ms\\nthread={:?}",
                        profile.exec_time.as_secs_f64() * 1000.0,
                        profile.queue_time.as_secs_f64() * 1000.0,
                        profile.thread_id,
                    ));
                }

                s.push_str(&format!(
                    "    {} [label=\"{}\", fillcolor=\"{}\"];\n",
                    node_idx.index(),
                    label,
                    color,
                ));
            }

            s.push_str("  }\n\n");
        }

        for edge in self.graph.edge_indices() {
            let (src, dst) = self.graph.edge_endpoints(edge).unwrap();
            s.push_str(&format!("  {} -> {};\n", src.index(), dst.index()));
        }

        s.push_str("}\n");

        s
    }
}

#[cfg(test)]
mod tests {
    use crate::TaskGraph;
    use std::time::Duration;

    #[test]
    fn test_sink() {
        let mut g = TaskGraph::new();

        let a = g.source("source", || Ok(2u32));
        let b = g.map("b", a, |x: &u32| Ok(x + 1));
        let c = g.map("c", b, |x: &u32| Ok(format!("value = {}", x)));

        let _ = g.sink("d", c, |s: &String| {
            assert_eq!("value = 3", s);
            Ok(())
        });

        g.execute(4).unwrap();
    }

    #[test]
    fn test_join_raw() {
        let mut g = TaskGraph::new();

        let a = g.source("source1", || Ok(1u32));
        let b = g.source("source2", || Ok(2u32));
        let c = g.source("source3", || Ok(3u32));

        let _ = g.join_raw("d", [a.raw, b.raw, c.raw], |(x, y, z): (u32, u32, u32)| {
            assert_eq!(1, x);
            assert_eq!(2, y);
            assert_eq!(3, z);
            Ok(())
        });
        g.execute(4).unwrap();
    }

    #[test]
    fn test_joins_raw() {
        let mut g = TaskGraph::new();

        let a = g.source("source1", || Ok(1u32));
        let b = g.source("source2", || Ok(2u32));
        let c = g.source("source3", || Ok(3u32));

        let _ = g.joins_raw("d", vec![a.raw, b.raw, c.raw], |inputs| {
            let x = *inputs[0].downcast_ref::<u32>().unwrap();
            let y = *inputs[1].downcast_ref::<u32>().unwrap();
            let z = *inputs[2].downcast_ref::<u32>().unwrap();
            assert_eq!(1, x);
            assert_eq!(2, y);
            assert_eq!(3, z);
            Ok(())
        });
        g.execute(4).unwrap();
    }

    #[test]
    fn test_joins() {
        let mut g = TaskGraph::new();

        let a = g.source("source1", || Ok(1u32));
        let b = g.source("source2", || Ok(2u32));
        let c = g.source("source3", || Ok(3u32));

        let _ = g.joins("d", vec![a, b, c], |inputs| {
            let x = *inputs[0].downcast_ref::<u32>().unwrap();
            let y = *inputs[1].downcast_ref::<u32>().unwrap();
            let z = *inputs[2].downcast_ref::<u32>().unwrap();
            assert_eq!(1, x);
            assert_eq!(2, y);
            assert_eq!(3, z);
            Ok(())
        });
        g.execute(4).unwrap();
    }

    #[test]
    fn test_join() {
        let mut g = TaskGraph::new();

        let a = g.source("source1", || Ok(1u32));
        let b = g.source("source2", || Ok(2u32));
        let c = g.source("source3", || Ok(3u32));

        let _ = g.join("d", [a, b, c], |(x, y, z): (u32, u32, u32)| {
            assert_eq!(1, x);
            assert_eq!(2, y);
            assert_eq!(3, z);
            Ok(())
        });
        g.execute(4).unwrap();
    }

    #[test]
    fn test_remove_task() {
        let mut g = TaskGraph::new();
        let a = g.source("source", || Ok(10));
        let b = g.map("b", a, |x| Ok(x + 1));
        let c = g.map("c", a, |x| Ok(x + 1));
        let d = g.map("d", b, |x| Ok(x + 1));
        let _ = g.join("e", vec![b, c, d], |(b, c, d): (i32, i32, i32)| {
            Ok(b + c + d + 1)
        });
        let _ = g.joins_raw("f", vec![a.raw, b.raw, c.raw], |_| Ok(1));
        let removed = g.remove_task(a.raw);
        assert_eq!(removed.len(), 6);
        g.execute(4).unwrap();
        let empty = "digraph TaskGraph {
  rankdir=LR;
  node [shape=box, style=filled, fontname=\"Consolas\"];

}
";
        assert_eq!(g.to_dot(), empty);
    }

    #[test]
    fn test_parallel() {
        let mut g = TaskGraph::new();
        let a = g.source("source", || {
            println!("generate source number, {:?}", std::thread::current().id());
            Ok(10)
        });

        let b = g.map("b", a, |x| {
            println!("map executed 1, {:?}", std::thread::current().id());
            std::thread::sleep(Duration::from_secs_f32(2.5));
            Ok(x + 1)
        });

        let c = g.map("c", a, |x| {
            println!("map executed 2, {:?}", std::thread::current().id());
            std::thread::sleep(Duration::from_secs_f32(2.5));
            Ok(x + 1)
        });

        let d = g.map("d", b, |x| {
            println!("map executed 3, {:?}", std::thread::current().id());
            std::thread::sleep(Duration::from_secs_f32(0.5));
            Ok(x + 1)
        });

        let _ = g.join("e", vec![b, c, d], |(b, c, d): (i32, i32, i32)| {
            println!("join executed 4, {:?}", std::thread::current().id());
            Ok(b + c + d + 1)
        });

        let _ = g.joins_raw("f", vec![a.raw, b.raw, c.raw], |_| {
            println!("joins_raw executed 5, {:?}", std::thread::current().id());
            Ok(1)
        });

        g.execute(4).unwrap();

        println!("\n{}", g.to_dot());
    }
}
