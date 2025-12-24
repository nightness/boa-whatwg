use std::time::Instant;

/// Represents the limits of different runtime operations.
#[derive(Debug, Clone, Copy)]
pub struct RuntimeLimits {
    /// Max stack size before an error is thrown.
    stack_size: usize,

    /// Max loop iterations before an error is thrown.
    loop_iteration: u64,

    /// Max backtrace count in exception.
    backtrace_limit: usize,

    /// Max function recursion limit
    recursion: usize,

    /// Execution deadline - if set, execution will stop when this instant is reached.
    execution_deadline: Option<Instant>,
}

impl Default for RuntimeLimits {
    #[inline]
    fn default() -> Self {
        Self {
            loop_iteration: u64::MAX,
            recursion: 512,
            backtrace_limit: 50,
            stack_size: 1024 * 10,
            execution_deadline: None,
        }
    }
}

impl RuntimeLimits {
    /// Return the loop iteration limit.
    ///
    /// If the limit is exceeded in a loop it will throw and error.
    ///
    /// The limit value [`u64::MAX`] means that there is no limit.
    #[inline]
    #[must_use]
    pub const fn loop_iteration_limit(&self) -> u64 {
        self.loop_iteration
    }

    /// Set the loop iteration limit.
    ///
    /// If the limit is exceeded in a loop it will throw and error.
    ///
    /// Setting the limit to [`u64::MAX`] means that there is no limit.
    #[inline]
    pub fn set_loop_iteration_limit(&mut self, value: u64) {
        self.loop_iteration = value;
    }

    /// Disable loop iteration limit.
    #[inline]
    pub fn disable_loop_iteration_limit(&mut self) {
        self.loop_iteration = u64::MAX;
    }

    /// Get max backtrace limit for an exception.
    ///
    /// Default is 50.
    #[inline]
    #[must_use]
    pub const fn backtrace_limit(&self) -> usize {
        self.backtrace_limit
    }

    /// Set max backtrace limit for an exception.
    #[inline]
    pub fn set_backtrace_limit(&mut self, value: usize) {
        self.backtrace_limit = value;
    }

    /// Get max stack size.
    #[inline]
    #[must_use]
    pub const fn stack_size_limit(&self) -> usize {
        self.stack_size
    }

    /// Set max stack size before an error is thrown.
    #[inline]
    pub fn set_stack_size_limit(&mut self, value: usize) {
        self.stack_size = value;
    }

    /// Get recursion limit.
    #[inline]
    #[must_use]
    pub const fn recursion_limit(&self) -> usize {
        self.recursion
    }

    /// Set recursion limit before an error is thrown.
    #[inline]
    pub fn set_recursion_limit(&mut self, value: usize) {
        self.recursion = value;
    }

    /// Get execution deadline.
    ///
    /// Returns `None` if no deadline is set.
    #[inline]
    #[must_use]
    pub const fn execution_deadline(&self) -> Option<Instant> {
        self.execution_deadline
    }

    /// Set execution deadline.
    ///
    /// If the deadline is reached during execution, a timeout error will be thrown.
    #[inline]
    pub fn set_execution_deadline(&mut self, deadline: Instant) {
        self.execution_deadline = Some(deadline);
    }

    /// Clear execution deadline.
    #[inline]
    pub fn clear_execution_deadline(&mut self) {
        self.execution_deadline = None;
    }

    /// Check if execution deadline has been exceeded.
    ///
    /// Returns `true` if the deadline has been exceeded, `false` otherwise.
    #[inline]
    #[must_use]
    pub fn is_deadline_exceeded(&self) -> bool {
        self.execution_deadline
            .map(|deadline| Instant::now() >= deadline)
            .unwrap_or(false)
    }
}
