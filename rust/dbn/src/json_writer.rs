#![allow(dead_code)]

// Re-export for version and casing consistency
pub use json_writer::{
    JSONArrayWriter as JsonArrayWriter, JSONObjectWriter as JsonObjectWriter,
    JSONStringWriter as JsonStringWriter, JSONWriter as JsonWriter,
    JSONWriterValue as JsonWriterValue, PrettyJSONWriter as PrettyJsonWriter, NULL,
};

/// Where a JSON value goes: after an object key or as the next array element.
pub trait Slot {
    type Writer: JsonWriter;

    /// Writes the key or the separator, leaving the writer positioned for the value.
    fn begin(&mut self) -> &mut Self::Writer;
}

/// The value of `key` in `object`.
pub struct Key<'o, 'w, 'k, J: JsonWriter> {
    pub object: &'o mut JsonObjectWriter<'w, J>,
    pub key: &'k str,
}

impl<J: JsonWriter> Slot for Key<'_, '_, '_, J> {
    type Writer = J;

    fn begin(&mut self) -> &mut J {
        self.object.write_key(self.key);
        self.object.writer
    }
}

impl<J: JsonWriter> Slot for JsonArrayWriter<'_, J> {
    type Writer = J;

    fn begin(&mut self) -> &mut J {
        self.write_comma();
        self.writer
    }
}
