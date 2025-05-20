//! Types related to task management

use super::TaskContext;
use crate::config::MAX_SYSCALL_ID;

/// Task information
///
/// How many syscalls a task has called, etc.
#[derive(Copy, Clone)]
pub struct TaskInfo {
    /// The counter of syscalls
    pub syscall_counter: [usize; MAX_SYSCALL_ID],
}

/// The task control block (TCB) of a task.
#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    /// The task status in it's lifecycle
    pub task_status: TaskStatus,
    /// The task context
    pub task_cx: TaskContext,
    /// The task information
    pub task_info: TaskInfo,
}

/// The status of a task
#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    /// uninitialized
    UnInit,
    /// ready to run
    Ready,
    /// running
    Running,
    /// exited
    Exited,
}

impl TaskControlBlock {
    /// Read and write task info
    pub fn read_task_syscall_counter(&self, id: usize) -> usize {
        self.task_info.syscall_counter[id]
    }

    /// Increase the counter of syscall id by 1
    pub fn increase_task_syscall_counter(&mut self, id: usize) {
        self.task_info.syscall_counter[id] += 1;
    }
}
