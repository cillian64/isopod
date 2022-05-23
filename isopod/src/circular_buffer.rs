/// Really simple circular buffer implementation
pub struct CircularBuffer<T> {
    // Buffer of readings
    buffer: Vec<T>,

    // The next position to write in the circular buffer
    ptr: usize,

    // The length of this circular buffer (once it is fully populated)
    capacity: usize,
}

impl<T> CircularBuffer<T>
where
    T: std::iter::Sum + Clone + std::ops::Div<f32, Output = T>,
{
    /// Create a new circular buffer with the specified capacity
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0);
        Self {
            buffer: Vec::new(),
            ptr: 0,
            capacity,
        }
    }

    /// Push a new element into the circular buffer
    pub fn push(&mut self, value: T) {
        if self.buffer.len() < self.capacity {
            self.buffer.push(value);
        } else {
            self.buffer[self.ptr] = value;
            self.ptr = (self.ptr + 1) % self.capacity;
        }
    }

    /// If the circular buffer is full then get an immutable reference to the
    /// contained buffer.  If the circular buffer is not yet full then None is
    /// returned
    #[allow(unused)]
    pub fn buffer(&self) -> Option<&Vec<T>> {
        if self.buffer.len() == self.capacity {
            Some(&self.buffer)
        } else {
            None
        }
    }

    /// If the circular buffer is full then return the mean of the contained
    /// values.  If it is not yet full then returns None.  Useful for
    /// implementing moving averages
    pub fn mean(&self) -> Option<T> {
        // Making this a method on the circular buffer was probably a really
        // bad idea, because it means we need a bunch of silly constraints on
        // the contained value type.  Instead the caller should just use
        // circ_buf.buffer().map(|buf| buf.iter().cloned().sum() / cap)
        if self.buffer.len() == self.capacity {
            Some(self.buffer.iter().cloned().sum::<T>() / (self.capacity as f32))
        } else {
            None
        }
    }

    /// If the circular buffer is full then return the most recently written
    /// value, otherwise return None
    pub fn head(&self) -> Option<T> {
        if self.buffer.len() == self.capacity {
            let ptr = if self.ptr == 0 {
                self.capacity - 1
            } else {
                self.ptr - 1
            };
            Some(self.buffer[ptr].clone())
        } else {
            None
        }
    }

    /// If the circular buffer is full then return the oldest value,
    /// otherwise return None
    pub fn tail(&self) -> Option<T> {
        if self.buffer.len() == self.capacity {
            Some(self.buffer[self.ptr].clone())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_circular_buffer() {
        let mut buf: CircularBuffer<f32> = CircularBuffer::new(5);

        buf.push(1.0);
        buf.push(2.0);
        buf.push(3.0);
        buf.push(4.0);

        // Check that all the getters return None when the buffer is not full
        assert_eq!(buf.buffer(), None);
        assert_eq!(buf.head(), None);
        assert_eq!(buf.tail(), None);
        assert_eq!(buf.mean(), None);

        buf.push(5.0);

        // Check the getters work now
        assert_eq!(buf.buffer(), Some(&vec![1.0, 2.0, 3.0, 4.0, 5.0]));
        assert_eq!(buf.head(), Some(5.0));
        assert_eq!(buf.tail(), Some(1.0));
        assert_eq!(buf.mean(), Some(3.0));

        // Check that the buffer is actually circular
        buf.push(6.0);
        buf.push(7.0);
        buf.push(8.0);
        assert_eq!(buf.buffer(), Some(&vec![6.0, 7.0, 8.0, 4.0, 5.0]));
        assert_eq!(buf.head(), Some(8.0));
        assert_eq!(buf.tail(), Some(4.0));
        assert_eq!(buf.mean(), Some(6.0));
    }
}
