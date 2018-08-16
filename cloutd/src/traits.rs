use Result;

/// A type that implements `Emitable` can be serialized.
pub trait Emitable {
    /// Return the length of the serialized data.
    fn buffer_len(&self) -> usize;

    /// Serialize this types and write the serialized data into the given buffer.
    ///
    /// # Panic
    ///
    /// This method panic if the buffer is not big enough. You **must** make sure the buffer is big
    /// enough before calling this method. You can use
    /// [`buffer_len()`](trait.Emitable.html#method.buffer_len) to check how big the storage needs
    /// to be.
    fn emit(&self, buffer: &mut [u8]);
}

/// A `Parseable` type can be used to deserialize data into the target type `T` for which it is
/// implemented.
pub trait Parseable<T> {
    /// Deserialize the current type.
    fn parse(&self) -> Result<T>;
}

impl<T: Emitable> Emitable for Option<T> {
    fn buffer_len(&self) -> usize {
        match *self {
            Some(ref v) => v.buffer_len(),
            None => 0,
        }
    }

    fn emit(&self, buffer: &mut [u8]) {
        if let Some(ref v) = *self {
            v.emit(buffer)
        }
    }
}

impl<T: Emitable> Emitable for Vec<T> {
    fn buffer_len(&self) -> usize {
        self.iter().fold(0usize, |sum, e| sum + e.buffer_len())
    }

    fn emit(&self, buffer: &mut [u8]) {
        self.iter().fold(buffer, |buf, e| {
            e.emit(buf);
            &mut buf[e.buffer_len()..]
        });
    }
}
