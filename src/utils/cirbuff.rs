use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct CircularBuffer<T, const C: usize> {
    buf: VecDeque<T>,
}

impl<T, const C: usize> Iterator for CircularBuffer<T, C> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.buf.pop_front()
    }
}

impl<T, const C: usize> Default for CircularBuffer<T, C> {
    fn default() -> Self {
        Self {
            buf: VecDeque::with_capacity(C),
        }
    }
}

impl<T, const C: usize> CircularBuffer<T, C> {
    pub fn push(&mut self, item: T) {
        if self.buf.len() == C {
            self.buf.pop_front(); // overwrite oldest
        }

        self.buf.push_back(item);
    }
}
