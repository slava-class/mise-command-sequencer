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
            if enabled {
                // If enabling this task for this step, clear any existing task for this step
                for (_, other_steps) in self.task_steps.iter_mut() {
                    if step < other_steps.len() {
                        other_steps[step] = false;
                    }
                }
            }

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

    pub fn generate_mise_task_command(&self) -> Option<String> {
        let mut commands = Vec::new();

        for step in 0..self.num_steps {
            let tasks_for_step = self.get_tasks_for_step(step);
            if !tasks_for_step.is_empty() {
                // Should only be one task per step based on current logic
                if let Some(task_name) = tasks_for_step.first() {
                    commands.push(format!("mise run {task_name}"));
                }
            }
        }

        if commands.is_empty() {
            None
        } else {
            Some(commands.join(" && "))
        }
    }
}

#[derive(Debug, Clone)]
pub enum SequenceEvent {
    ToggleStep(String, usize),
    RunSequence,
    CopyAsTask,
    ClearSequence,
    StepCompleted,
    SequenceCompleted,
    SequenceFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequence_state_new() {
        let seq = SequenceState::new(3);
        assert_eq!(seq.num_steps, 3);
        assert_eq!(seq.task_steps.len(), 0);
        assert_eq!(seq.current_step, None);
        assert!(!seq.is_running);
        assert_eq!(seq.completed_steps, vec![false, false, false]);
    }

    #[test]
    fn test_set_and_get_task_step() {
        let mut seq = SequenceState::new(3);

        seq.set_task_step("build", 0, true);
        seq.set_task_step("test", 1, true);
        seq.set_task_step("build", 2, true);

        assert!(seq.is_task_enabled_for_step("build", 0));
        assert!(!seq.is_task_enabled_for_step("build", 1));
        assert!(seq.is_task_enabled_for_step("build", 2));
        assert!(seq.is_task_enabled_for_step("test", 1));
        assert!(!seq.is_task_enabled_for_step("test", 0));
    }

    #[test]
    fn test_set_task_step_bounds() {
        let mut seq = SequenceState::new(2);

        seq.set_task_step("build", 5, true); // Out of bounds
        assert!(!seq.is_task_enabled_for_step("build", 5));
        assert_eq!(seq.task_steps.len(), 0); // Should not create entry
    }

    #[test]
    fn test_is_task_enabled_for_nonexistent_task() {
        let seq = SequenceState::new(3);
        assert!(!seq.is_task_enabled_for_step("nonexistent", 0));
    }

    #[test]
    fn test_get_tasks_for_step() {
        let mut seq = SequenceState::new(3);

        seq.set_task_step("build", 0, true);
        seq.set_task_step("test", 0, true); // This should replace "build" for step 0
        seq.set_task_step("deploy", 1, true);

        let step0_tasks = seq.get_tasks_for_step(0);
        assert_eq!(step0_tasks.len(), 1); // Only one task allowed per step
        assert!(step0_tasks.contains(&"test".to_string())); // "test" should replace "build"

        let step1_tasks = seq.get_tasks_for_step(1);
        assert_eq!(step1_tasks.len(), 1);
        assert!(step1_tasks.contains(&"deploy".to_string()));

        let step2_tasks = seq.get_tasks_for_step(2);
        assert_eq!(step2_tasks.len(), 0);
    }

    #[test]
    fn test_one_task_per_step() {
        let mut seq = SequenceState::new(3);

        // Enable first task for step 0
        seq.set_task_step("build", 0, true);
        assert!(seq.is_task_enabled_for_step("build", 0));

        // Enable second task for step 0 - should replace first task
        seq.set_task_step("test", 0, true);
        assert!(!seq.is_task_enabled_for_step("build", 0)); // build should be disabled
        assert!(seq.is_task_enabled_for_step("test", 0)); // test should be enabled

        // Enable same task again - should toggle it off
        seq.set_task_step("test", 0, false);
        assert!(!seq.is_task_enabled_for_step("test", 0));

        // Enable different task in different step - should not affect other steps
        seq.set_task_step("deploy", 1, true);
        assert!(seq.is_task_enabled_for_step("deploy", 1));
        assert!(!seq.is_task_enabled_for_step("test", 0));
    }

    #[test]
    fn test_clear_all() {
        let mut seq = SequenceState::new(3);

        seq.set_task_step("build", 0, true);
        seq.set_task_step("test", 1, true);
        seq.start_execution();

        seq.clear_all();

        assert!(!seq.is_task_enabled_for_step("build", 0));
        assert!(!seq.is_task_enabled_for_step("test", 1));
        assert_eq!(seq.current_step, None);
        assert!(!seq.is_running);
        assert_eq!(seq.completed_steps, vec![false, false, false]);
    }

    #[test]
    fn test_reset_execution() {
        let mut seq = SequenceState::new(3);
        seq.start_execution();
        seq.completed_steps[0] = true;

        seq.reset_execution();

        assert_eq!(seq.current_step, None);
        assert!(!seq.is_running);
        assert_eq!(seq.completed_steps, vec![false, false, false]);
    }

    #[test]
    fn test_start_execution() {
        let mut seq = SequenceState::new(3);

        seq.start_execution();

        assert_eq!(seq.current_step, Some(0));
        assert!(seq.is_running);
        assert_eq!(seq.completed_steps, vec![false, false, false]);
    }

    #[test]
    fn test_advance_step() {
        let mut seq = SequenceState::new(3);
        seq.start_execution();

        // Advance from step 0 to 1
        let has_more = seq.advance_step();
        assert!(has_more);
        assert_eq!(seq.current_step, Some(1));
        assert!(seq.is_running);
        assert_eq!(seq.completed_steps[0], true);
        assert_eq!(seq.completed_steps[1], false);

        // Advance from step 1 to 2
        let has_more = seq.advance_step();
        assert!(has_more);
        assert_eq!(seq.current_step, Some(2));
        assert!(seq.is_running);
        assert_eq!(seq.completed_steps[1], true);

        // Advance from step 2 (final step)
        let has_more = seq.advance_step();
        assert!(!has_more);
        assert_eq!(seq.current_step, None);
        assert!(!seq.is_running);
        assert_eq!(seq.completed_steps[2], true);
    }

    #[test]
    fn test_advance_step_when_not_running() {
        let mut seq = SequenceState::new(3);

        let has_more = seq.advance_step();
        assert!(!has_more);
        assert_eq!(seq.current_step, None);
        assert!(!seq.is_running);
    }

    #[test]
    fn test_generate_mise_task_command_with_tasks() {
        let mut seq = SequenceState::new(3);

        seq.set_task_step("build", 0, true);
        seq.set_task_step("test", 1, true);
        seq.set_task_step("deploy", 2, true);

        let command = seq.generate_mise_task_command();
        assert_eq!(
            command,
            Some("mise run build && mise run test && mise run deploy".to_string())
        );
    }

    #[test]
    fn test_generate_mise_task_command_with_gaps() {
        let mut seq = SequenceState::new(3);

        seq.set_task_step("build", 0, true);
        // Skip step 1
        seq.set_task_step("deploy", 2, true);

        let command = seq.generate_mise_task_command();
        assert_eq!(
            command,
            Some("mise run build && mise run deploy".to_string())
        );
    }

    #[test]
    fn test_generate_mise_task_command_empty() {
        let seq = SequenceState::new(3);

        let command = seq.generate_mise_task_command();
        assert_eq!(command, None);
    }

    #[test]
    fn test_generate_mise_task_command_single_task() {
        let mut seq = SequenceState::new(3);

        seq.set_task_step("build", 1, true);

        let command = seq.generate_mise_task_command();
        assert_eq!(command, Some("mise run build".to_string()));
    }
}
