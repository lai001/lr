#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TaskKind {
    Source,
    Map,
    Sink,
    Join,
}

impl std::fmt::Display for TaskKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskKind::Source => write!(f, "source"),
            TaskKind::Map => write!(f, "map"),
            TaskKind::Sink => write!(f, "sink"),
            TaskKind::Join => write!(f, "join"),
        }
    }
}
