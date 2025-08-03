#[derive(Debug)]
pub enum AppState {
    List,
    Detail(String),
    Running(String),
    SequenceBuilder,
}
