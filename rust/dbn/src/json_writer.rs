#![allow(dead_code)]

// Re-export for version and casing consistency
pub use json_writer::{
    JSONArrayWriter as JsonArrayWriter, JSONObjectWriter as JsonObjectWriter, JSONWriter,
    JSONWriter as JsonWriter, JSONWriterValue as JsonWriterValue,
    PrettyJSONWriter as PrettyJsonWriter, NULL,
};
