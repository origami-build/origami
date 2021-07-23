use crate::project::Project;
use crate::task::Task;

pub trait Plugin {
    fn create_tasks(&self, project: &Project) -> Vec<Box<dyn Task>>;
}
