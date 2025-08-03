use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SequenceState {
    pub task_steps: HashMap<String, Vec<bool>>,
    pub num_steps: usize,
    pub current_step: Option<usize>,
    pub is_running: bool,
    pub completed_steps: Vec<bool>,
}

impl SequenceState {
    pub fn new(num_steps: usize) -> Self {
        Self {
            task_steps: HashMap::new(),
            num_steps,
            current_step: None,
            is_running: false,
            completed_steps: vec![false; num_steps],
        }
    }

    pub fn set_task_step(&mut self, task_name: &str, step: usize, enabled: bool) {
        if step < self.num_steps {
            let steps = self
                .task_steps
                .entry(task_name.to_string())
                .or_insert_with(|| vec![false; self.num_steps]);
            if step < steps.len() {
                steps[step] = enabled;
            }
        }
    }

    pub fn is_task_enabled_for_step(&self, task_name: &str, step: usize) -> bool {
        self.task_steps
            .get(task_name)
            .map(|steps| step < steps.len() && steps[step])
            .unwrap_or(false)
    }

    pub fn get_tasks_for_step(&self, step: usize) -> Vec<String> {
        self.task_steps
            .iter()
            .filter_map(|(task_name, steps)| {
                if step < steps.len() && steps[step] {
                    Some(task_name.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn clear_all(&mut self) {
        for steps in self.task_steps.values_mut() {
            steps.fill(false);
        }
        self.reset_execution();
    }

    pub fn reset_execution(&mut self) {
        self.current_step = None;
        self.is_running = false;
        self.completed_steps.fill(false);
    }

    pub fn start_execution(&mut self) {
        self.current_step = Some(0);
        self.is_running = true;
        self.completed_steps.fill(false);
    }

    pub fn advance_step(&mut self) -> bool {
        if let Some(current) = self.current_step {
            self.completed_steps[current] = true;
            if current + 1 < self.num_steps {
                self.current_step = Some(current + 1);
                true
            } else {
                self.current_step = None;
                self.is_running = false;
                false
            }
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub enum SequenceEvent {
    ToggleStep(String, usize),
    RunSequence,
    ClearSequence,
    StepCompleted,
    SequenceCompleted,
    SequenceFailed(String),
}
