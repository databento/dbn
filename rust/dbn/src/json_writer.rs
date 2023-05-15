#![allow(dead_code)]

///
/// Helper for appending a JSON object to the borrowed buffer.
///
/// Appends '{' on creation.
/// Appends '}' when dropped.
///
pub struct JsonObjectWriter<'a, F: Formatter> {
    ///
    /// Mutable borrow of buffer
    ///
    /// Consider using the methods instead of using this field directly.
    /// This field should not be used unless you know what you are doing.
    ///
    pub buffer: &'a mut String,
    empty: bool,
    formatter: F,
}

///
/// Helper for appending a JSON array to the borrowed buffer.
///
/// Appends '[' on creation.
/// Appends ']' when dropped.
///
pub struct JsonArrayWriter<'a, F: Formatter> {
    ///
    /// Mutable borrow of buffer
    ///
    /// Consider using the methods instead of using this field directly.
    /// This field should not be used unless you know what you are doing.
    ///
    pub buffer: &'a mut String,
    empty: bool,
    formatter: F,
}

#[doc(hidden)]
#[derive(Debug, Copy, Clone)]
pub struct Null();

///
/// Represents the null value in json.
///
/// **Note**: Option::None may be used instead in most cases.
///
pub static NULL: Null = Null();

impl JsonObjectWriter<'_, CompactFormatter> {
    ///
    /// Creates a new JsonObjectWriter that writes to the given buffer. Writes '{' to the buffer immediately.
    /// Uses [`CompactFormatter`].
    ///
    #[inline(always)]
    pub fn new(buffer: &mut String) -> JsonObjectWriter<CompactFormatter> {
        JsonObjectWriter::with_formatter(buffer, CompactFormatter {})
    }
}

impl<F: Formatter> JsonObjectWriter<'_, F> {
    /// Creates a new writer with the specified `formatter`.
    pub fn with_formatter(buffer: &mut String, mut formatter: F) -> JsonObjectWriter<'_, F> {
        formatter.begin_object(buffer);
        JsonObjectWriter {
            buffer,
            empty: true,
            formatter,
        }
    }

    ///
    /// Starts writing a nested object with given key:
    ///
    /// Esacapes key, writes "\"key\":{" and returns a JsonObjectWriter
    ///
    #[inline(always)]
    pub fn object(&mut self, key: &str) -> JsonObjectWriter<'_, F> {
        self.write_key(key);
        Self::with_formatter(self.buffer, self.formatter.clone())
    }

    ///
    /// Starts writing a nested array with given key:
    ///
    /// Esacapes key, writes "\"key\":[" and returns a JsonArrayWriter.
    ///
    #[inline(always)]
    pub fn array(&mut self, key: &str) -> JsonArrayWriter<'_, F> {
        self.write_key(key);
        JsonArrayWriter::with_formatter(self.buffer, self.formatter.clone())
    }

    ///
    /// Escapes and appends key:value to the buffer
    ///
    #[inline(always)]
    pub fn value<T: JsonWriterValue>(&mut self, key: &str, value: T) {
        self.write_key(key);
        value.write_json(self.buffer, &self.formatter);
    }

    ///
    /// Writes a key without any value.
    ///
    /// Consider using the methods value(key, value), object(key) and array(key) instead of using this method directly.
    ///
    /// <p style="background:rgba(255,181,77,0.16);padding:0.75em;">
    /// <strong>Warning:</strong>
    /// If you use this method, you will have to write the value to the buffer youself afterwards.
    /// </p>
    ///
    #[inline(never)]
    pub fn write_key(&mut self, key: &str) {
        self.formatter.write_comma(self.buffer, self.empty);
        self.empty = false;
        write_string(self.buffer, key);
        self.formatter.write_kv_colon(self.buffer);
    }

    ///
    /// Drops the writer.
    /// Dropping causes '}' to be appended to the buffer.
    ///
    #[inline(always)]
    pub fn end(self) {
        drop(self);
    }

    ///
    /// Writes the entire buffer to given writer and clears entire buffer on success.
    ///
    #[inline(always)]
    pub fn output_buffered_data<Writer: std::io::Write>(
        &mut self,
        writer: &mut Writer,
    ) -> Result<usize, std::io::Error> {
        output_buffer_to(self.buffer, writer)
    }

    ///
    /// Returns buffer length in bytes
    ///
    #[inline(always)]
    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }
}

impl<F: Formatter> Drop for JsonObjectWriter<'_, F> {
    #[inline(always)]
    fn drop(&mut self) {
        self.formatter.end_object(self.buffer, self.empty)
    }
}

impl JsonArrayWriter<'_, CompactFormatter> {
    ///
    /// Creates a new JsonArrayWriter that writes to the given buffer. Writes '[' to the buffer immediately.
    /// Uses [`CompactFormatter`].
    ///
    #[inline(always)]
    pub fn new(buffer: &mut String) -> JsonArrayWriter<'_, CompactFormatter> {
        JsonArrayWriter::with_formatter(buffer, CompactFormatter {})
    }
}

impl<F: Formatter> JsonArrayWriter<'_, F> {
    /// Creates a new writer with the specified `formatter`.
    pub fn with_formatter(buffer: &mut String, mut formatter: F) -> JsonArrayWriter<'_, F> {
        formatter.begin_array(buffer);
        JsonArrayWriter {
            buffer,
            empty: true,
            formatter,
        }
    }

    ///
    /// Starts writing a nested object as array entry.
    ///
    /// Writes '{' and returns a JsonObjectWriter
    ///
    #[inline(always)]
    pub fn object(&mut self) -> JsonObjectWriter<'_, F> {
        self.write_comma();
        JsonObjectWriter::with_formatter(self.buffer, self.formatter.clone())
    }

    ///
    /// Starts writing a nested array as array entry.
    ///
    /// Writes '[' and returns a JsonArrayWriter
    ///
    #[inline(always)]
    pub fn array(&mut self) -> JsonArrayWriter<'_, F> {
        self.write_comma();
        JsonArrayWriter::with_formatter(self.buffer, self.formatter.clone())
    }

    ///
    /// Writes given value as array entry
    ///
    #[inline(always)]
    pub fn value<T: JsonWriterValue>(&mut self, value: T) {
        self.write_comma();
        value.write_json(self.buffer, &self.formatter);
    }

    ///
    /// Writes a comma unless at the beginning of the array
    ///
    /// <p style="background:rgba(255,181,77,0.16);padding:0.75em;">
    /// <strong>Warning:</strong>
    /// If you use this method, you will have to write the value to the buffer youself afterwards.
    /// </p>
    ///
    pub fn write_comma(&mut self) {
        self.formatter.write_comma(self.buffer, self.empty);
        self.empty = false;
    }

    ///
    /// Drops the writer.
    /// Dropping causes ']' to be appended to the buffer.
    ///
    #[inline(always)]
    pub fn end(self) {
        drop(self)
    }

    ///
    /// Writes the entire buffer to given writer and clears entire buffer on success.
    ///
    #[inline(always)]
    pub fn output_buffered_data<Writer: std::io::Write>(
        &mut self,
        writer: &mut Writer,
    ) -> Result<usize, std::io::Error> {
        output_buffer_to(self.buffer, writer)
    }

    ///
    /// Returns buffer length in bytes
    ///
    #[inline(always)]
    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }
}

impl<F: Formatter> Drop for JsonArrayWriter<'_, F> {
    #[inline(always)]
    fn drop(&mut self) {
        self.formatter.end_array(self.buffer, self.empty)
    }
}

/// This trait allows control around optional whitespace when writing JSON.
pub trait Formatter: Clone {
    /// Called at the start of writing an object.
    fn begin_object(&mut self, buffer: &mut String) {
        buffer.push('{');
    }

    /// Called after writing all key-value pairs of an object.
    ///
    /// `empty` is `true` when the object contains no key-value pairs.
    fn end_object(&mut self, buffer: &mut String, _empty: bool) {
        buffer.push('}');
    }

    /// Called at the start of writing an array.
    fn begin_array(&mut self, buffer: &mut String) {
        buffer.push('[');
    }

    /// Called after writing all items of an array.
    ///
    /// `empty` is `true` when the array contains no items.
    fn end_array(&mut self, buffer: &mut String, _empty: bool) {
        buffer.push(']');
    }

    /// Called before each key-value pair in an object and each item in an array.
    ///
    /// `empty` is `true` when it's the first key-value pair or item.
    fn write_comma(&mut self, buffer: &mut String, empty: bool) {
        if !empty {
            buffer.push(',');
        }
    }

    /// Called between the key and value.
    fn write_kv_colon(&mut self, buffer: &mut String) {
        buffer.push(':');
    }
}

/// Formats JSON as compactly as possible.
#[derive(Clone, Copy, Default)]
pub struct CompactFormatter {}

impl Formatter for CompactFormatter {}

/// Formats JSON in a human-readable format with whitespace, newlines, and indentation.
#[derive(Clone)]
pub struct PrettyFormatter<'a> {
    indent: &'a str,
    depth: usize,
}

impl Default for PrettyFormatter<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl PrettyFormatter<'_> {
    /// Creates a new human-readable formatter with two spaces for indentation.
    pub const fn new() -> Self {
        // Same default as serde_json::ser::PrettyFormatter
        Self {
            indent: "  ",
            depth: 0,
        }
    }

    /// Creates a new formatter using `indent` for indentation.
    pub const fn with_indent(indent: &str) -> PrettyFormatter<'_> {
        PrettyFormatter { indent, depth: 0 }
    }
}

impl<'a> Formatter for PrettyFormatter<'a> {
    fn begin_object(&mut self, buffer: &mut String) {
        self.depth += 1;
        buffer.push('{');
    }

    fn end_object(&mut self, buffer: &mut String, empty: bool) {
        self.depth -= 1;
        if !empty {
            buffer.push('\n');
            self.write_indent(buffer);
        }
        buffer.push('}');
    }

    fn begin_array(&mut self, buffer: &mut String) {
        self.depth += 1;
        buffer.push('[');
    }

    fn end_array(&mut self, buffer: &mut String, empty: bool) {
        self.depth -= 1;
        if !empty {
            buffer.push('\n');
            self.write_indent(buffer);
        }
        buffer.push(']');
    }

    fn write_comma(&mut self, buffer: &mut String, empty: bool) {
        if empty {
            buffer.push('\n')
        } else {
            buffer.push_str(",\n");
        }
        self.write_indent(buffer);
    }

    fn write_kv_colon(&mut self, buffer: &mut String) {
        buffer.push_str(": ");
    }
}

impl<'a> PrettyFormatter<'a> {
    fn write_indent(&self, buffer: &mut String) {
        for _ in 0..self.depth {
            buffer.push_str(self.indent);
        }
    }
}

///
/// Types with this trait can be converted to JSON
///
pub trait JsonWriterValue {
    ///
    /// Appends a JSON representation of self to the output buffer
    ///
    fn write_json<F: Formatter>(self, output_buffer: &mut String, formatter: &F);
}

impl JsonWriterValue for &str {
    #[inline(always)]
    fn write_json<F: Formatter>(self, output_buffer: &mut String, _formatter: &F) {
        write_string(output_buffer, self);
    }
}

impl JsonWriterValue for &std::borrow::Cow<'_, str> {
    #[inline(always)]
    fn write_json<F: Formatter>(self, output_buffer: &mut String, _formatter: &F) {
        write_string(output_buffer, std::convert::AsRef::as_ref(self));
    }
}

impl JsonWriterValue for &String {
    #[inline(always)]
    fn write_json<F: Formatter>(self, output_buffer: &mut String, _formatter: &F) {
        write_string(output_buffer, self);
    }
}

impl JsonWriterValue for f64 {
    #[inline(always)]
    fn write_json<F: Formatter>(self, output_buffer: &mut String, _formatter: &F) {
        write_float(output_buffer, self);
    }
}

impl JsonWriterValue for f32 {
    #[inline(always)]
    fn write_json<F: Formatter>(self, output_buffer: &mut String, _formatter: &F) {
        write_float(output_buffer, self as f64);
    }
}

impl JsonWriterValue for u32 {
    #[inline(always)]
    fn write_json<F: Formatter>(self, output_buffer: &mut String, _formatter: &F) {
        let mut buf = itoa::Buffer::new();
        output_buffer.push_str(buf.format(self));
    }
}

impl JsonWriterValue for i32 {
    #[inline(always)]
    fn write_json<F: Formatter>(self, output_buffer: &mut String, _formatter: &F) {
        let mut buf = itoa::Buffer::new();
        output_buffer.push_str(buf.format(self));
    }
}
impl JsonWriterValue for u16 {
    #[inline(always)]
    fn write_json<F: Formatter>(self, output_buffer: &mut String, _formatter: &F) {
        let mut buf = itoa::Buffer::new();
        output_buffer.push_str(buf.format(self));
    }
}

impl JsonWriterValue for i16 {
    #[inline(always)]
    fn write_json<F: Formatter>(self, output_buffer: &mut String, _formatter: &F) {
        let mut buf = itoa::Buffer::new();
        output_buffer.push_str(buf.format(self));
    }
}

impl JsonWriterValue for u8 {
    #[inline(always)]
    fn write_json<F: Formatter>(self, output_buffer: &mut String, _formatter: &F) {
        let mut buf = itoa::Buffer::new();
        output_buffer.push_str(buf.format(self));
    }
}

impl JsonWriterValue for i8 {
    #[inline(always)]
    fn write_json<F: Formatter>(self, output_buffer: &mut String, _formatter: &F) {
        let mut buf = itoa::Buffer::new();
        output_buffer.push_str(buf.format(self));
    }
}

impl JsonWriterValue for bool {
    #[inline(always)]
    fn write_json<F: Formatter>(self, output_buffer: &mut String, _formatter: &F) {
        output_buffer.push_str(if self { "true" } else { "false" });
    }
}

impl JsonWriterValue for Null {
    #[inline(always)]
    fn write_json<F: Formatter>(self, output_buffer: &mut String, _formatter: &F) {
        output_buffer.push_str("null");
    }
}

impl<T: JsonWriterValue + Copy> JsonWriterValue for &T {
    #[inline(always)]
    fn write_json<F: Formatter>(self, output_buffer: &mut String, formatter: &F) {
        (*self).write_json(output_buffer, formatter);
    }
}

impl<T: JsonWriterValue> JsonWriterValue for Option<T> {
    #[inline(always)]
    fn write_json<F: Formatter>(self, output_buffer: &mut String, formatter: &F) {
        match self {
            None => {
                output_buffer.push_str("null");
            }
            Some(value) => {
                value.write_json(output_buffer, formatter);
            }
        }
    }
}

impl<Item> JsonWriterValue for &Vec<Item>
where
    for<'b> &'b Item: JsonWriterValue,
{
    #[inline(always)]
    fn write_json<F: Formatter>(self, output_buffer: &mut String, formatter: &F) {
        self.as_slice().write_json(output_buffer, formatter);
    }
}

impl<Item> JsonWriterValue for &[Item]
where
    for<'b> &'b Item: JsonWriterValue,
{
    fn write_json<F: Formatter>(self, output_buffer: &mut String, formatter: &F) {
        let mut array = JsonArrayWriter::with_formatter(output_buffer, formatter.clone());
        for item in self.iter() {
            array.value(item);
        }
    }
}

impl<Key: AsRef<str>, Item> JsonWriterValue for &std::collections::HashMap<Key, Item>
where
    for<'b> &'b Item: JsonWriterValue,
{
    fn write_json<F: Formatter>(self, output_buffer: &mut String, formatter: &F) {
        let mut obj = JsonObjectWriter::with_formatter(output_buffer, formatter.clone());
        for (key, value) in self.iter() {
            obj.value(key.as_ref(), value);
        }
    }
}

impl<Key: AsRef<str>, Item> JsonWriterValue for &std::collections::BTreeMap<Key, Item>
where
    for<'b> &'b Item: JsonWriterValue,
{
    fn write_json<F: Formatter>(self, output_buffer: &mut String, formatter: &F) {
        let mut obj = JsonObjectWriter::with_formatter(output_buffer, formatter.clone());
        for (key, value) in self.iter() {
            obj.value(key.as_ref(), value);
        }
    }
}

///
/// Converts given value to a json string.
///
#[inline]
pub fn to_json_string<T: JsonWriterValue>(v: T) -> String {
    let mut result = String::new();
    v.write_json(&mut result, &CompactFormatter {});
    result
}

fn output_buffer_to<Writer: std::io::Write>(
    buffer: &mut String,
    writer: &mut Writer,
) -> Result<usize, std::io::Error> {
    match writer.write_all(buffer.as_bytes()) {
        Ok(_) => {
            let len = buffer.len();
            buffer.clear();
            Ok(len)
        }
        Err(err) => Err(err),
    }
}

///
/// Quotes and escapes input and appends result to output buffer
///
#[inline(never)]
pub fn write_string(output_buffer: &mut String, input: &str) {
    output_buffer.push('"');
    write_part_of_string_impl(output_buffer, input);
    output_buffer.push('"');
}

///
/// Escapes input and appends result to output buffer without adding quotes.
///
#[inline(never)]
pub fn write_part_of_string(output_buffer: &mut String, input: &str) {
    write_part_of_string_impl(output_buffer, input);
}

const fn get_replacements() -> [u8; 256] {
    // NOTE: Only characters smaller than 128 are allowed here.
    // Trying to escape values above 128 would generate invalid utf-8 output
    // -----
    // see https://www.json.org/json-en.html
    let mut result = [0u8; 256];
    // Escape everything from 0 to 0x1F
    let mut i = 0;
    while i < 0x20 {
        result[i] = b'u';
        i += 1;
    }
    result[b'\"' as usize] = b'"';
    result[b'\\' as usize] = b'\\';
    result[b'/' as usize] = b'/';
    result[8] = b'b';
    result[0xc] = b'f';
    result[b'\n' as usize] = b'n';
    result[b'\r' as usize] = b'r';
    result[b'\t' as usize] = b't';
    result[0] = b'u';
    result
}
static REPLACEMENTS: [u8; 256] = get_replacements();
static HEX: [u8; 16] = *b"0123456789ABCDEF";

///
/// Escapes and append part of string
///
#[inline(always)]
fn write_part_of_string_impl(output_buffer: &mut String, input: &str) {
    // All of the relevant characters are in the ansi range (<128).
    // This means we can safely ignore any utf-8 characters and iterate over the bytes directly
    let mut num_bytes_written: usize = 0;
    let mut index: usize = 0;
    let bytes = input.as_bytes();
    while index < bytes.len() {
        let cur_byte = bytes[index];
        let replacement = REPLACEMENTS[cur_byte as usize];
        if replacement != 0 {
            if num_bytes_written < index {
                // Checks can be ommitted here:
                // We know that index is smaller than the output_buffer length.
                // We also know that num_bytes_written is smaller than index
                // We also know that the boundaries are not in the middle of an utf-8 multi byte sequence, because those characters are not escaped
                output_buffer.push_str(unsafe { input.get_unchecked(num_bytes_written..index) });
            }
            if replacement == b'u' {
                let bytes: [u8; 6] = [
                    b'\\',
                    b'u',
                    b'0',
                    b'0',
                    HEX[((cur_byte / 16) & 0xF) as usize],
                    HEX[(cur_byte & 0xF) as usize],
                ];
                // Checks can be ommitted here: We know bytes is a valid utf-8 string (see above)
                output_buffer.push_str(unsafe { std::str::from_utf8_unchecked(&bytes) });
            } else {
                let bytes: [u8; 2] = [b'\\', replacement];
                // Checks can be ommitted here: We know bytes is a valid utf-8 string, because the replacement table only contains characters smaller than 128
                output_buffer.push_str(unsafe { std::str::from_utf8_unchecked(&bytes) });
            }
            num_bytes_written = index + 1;
        }
        index += 1;
    }
    if num_bytes_written < bytes.len() {
        // Checks can be ommitted here:
        // We know that num_bytes_written is smaller than index
        // We also know that num_bytes_written not in the middle of an utf-8 multi byte sequence, because those are not escaped
        output_buffer.push_str(unsafe { input.get_unchecked(num_bytes_written..bytes.len()) });
    }
}

///
/// If value is finite then value is converted to string and appended to buffer.
/// If value is NaN or infinity, then the string "null" is appended to buffer (without the quotes)
///
#[inline(never)]
pub fn write_float(output_buffer: &mut String, value: f64) {
    if !value.is_finite() {
        // JSON does not allow infinite or nan values. In browsers JSON.stringify(Number.NaN) = "null"
        output_buffer.push_str("null");
        return;
    }

    let mut buf = ryu::Buffer::new();
    let mut result = buf.format_finite(value);
    if result.ends_with(".0") {
        result = unsafe { result.get_unchecked(..result.len() - 2) };
    }
    output_buffer.push_str(result);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_array() {
        let mut buffer = String::new();
        let mut array = JsonArrayWriter::new(&mut buffer);
        array.value(0u8);
        array.value(1i32);
        array.value("2");
        array.value("\"<script>1/2</script>\"");
        let mut nested_arr = array.array();
        nested_arr.value("nested");
        nested_arr.end();
        let mut nested_obj = array.object();
        nested_obj.value("ä\töü", "ä\töü");
        nested_obj.end();
        let nested_obj2 = array.object();
        nested_obj2.end();
        drop(array);

        assert_eq!(buffer, "[0,1,\"2\",\"\\\"<script>1\\/2<\\/script>\\\"\",[\"nested\"],{\"ä\\töü\":\"ä\\töü\"},{}]");
    }

    #[test]
    fn test_array_range() {
        let bytes = b"ABC";
        assert_eq!(to_json_string(&bytes[..]), "[65,66,67]");

        let mut v = Vec::<u8>::new();
        v.extend_from_slice(bytes);
        assert_eq!(to_json_string(&v), "[65,66,67]");
    }

    #[test]
    fn test_object() {
        let mut map = std::collections::HashMap::<String, String>::new();
        map.insert("a".to_owned(), "a".to_owned());
        assert_eq!(to_json_string(&map), "{\"a\":\"a\"}");
    }

    #[test]
    fn test_numbers() {
        // unsigned
        assert_eq!(to_json_string(1u8), "1");
        assert_eq!(to_json_string(1u16), "1");
        assert_eq!(to_json_string(1u32), "1");
        assert_eq!(to_json_string(u8::MAX), "255");
        assert_eq!(to_json_string(u16::MAX), "65535");
        assert_eq!(to_json_string(u32::MAX), "4294967295");

        // signed
        assert_eq!(to_json_string(-1i8), "-1");
        assert_eq!(to_json_string(-1i16), "-1");
        assert_eq!(to_json_string(-1i32), "-1");

        // float
        assert_eq!(to_json_string(0f32), "0");
        assert_eq!(to_json_string(2f32), "2");
        assert_eq!(to_json_string(-2f32), "-2");

        assert_eq!(to_json_string(0f64), "0");
        assert_eq!(to_json_string(2f64), "2");
        assert_eq!(to_json_string(-2f64), "-2");
        assert_eq!(to_json_string(3.141592653589793), "3.141592653589793");
        assert_eq!(to_json_string(0.1f64), "0.1");
        assert_eq!(to_json_string(-0.1f64), "-0.1");
        //assert_eq!(to_json_string(-5.0/3.0), "-1.6666666666666667");
        assert_eq!(to_json_string(1.5e30f64), "1.5e30");
        assert_eq!(
            to_json_string(-2.220446049250313e-16f64),
            "-2.220446049250313e-16"
        );

        assert_eq!(to_json_string(1.0 / 0.0), "null");
        assert_eq!(to_json_string(std::f64::INFINITY), "null");
        assert_eq!(to_json_string(std::f64::NEG_INFINITY), "null");
        assert_eq!(to_json_string(std::f64::NAN), "null");
    }

    #[test]
    fn test_dtoa() {
        assert_dtoa(0.0);
        assert_dtoa(1.0);
        assert_dtoa(-1.0);
        assert_dtoa(2.0);
        //assert_dtoa(-5.0/3.0);
    }

    fn assert_dtoa(v: f64) {
        let a = v.to_string();
        let mut b = String::new();
        write_float(&mut b, v);
        assert_eq!(b, a);
    }

    #[test]
    fn test_strings() {
        assert_eq!(
            to_json_string("中文\0\x08\x09\"\\\n\r\t</script>"),
            "\"中文\\u0000\\b\\t\\\"\\\\\\n\\r\\t<\\/script>\""
        );
    }

    #[test]
    fn test_basic_example() {
        let mut object_str = String::new();
        {
            let mut object_writer = JsonObjectWriter::new(&mut object_str);
            object_writer.value("number", 42i32);
        }
        assert_eq!(&object_str, "{\"number\":42}");
    }

    #[test]
    fn test_misc_examples() {
        // Values
        assert_eq!(to_json_string("Hello World\n"), "\"Hello World\\n\"");
        assert_eq!(to_json_string(3.141592653589793f64), "3.141592653589793");
        assert_eq!(to_json_string(true), "true");
        assert_eq!(to_json_string(false), "false");
        assert_eq!(to_json_string(NULL), "null");

        // Options of values
        assert_eq!(to_json_string(Option::<u8>::Some(42)), "42");
        assert_eq!(to_json_string(Option::<u8>::None), "null");

        // Slices and vectors
        let numbers: [u8; 4] = [1, 2, 3, 4];
        assert_eq!(to_json_string(&numbers[..]), "[1,2,3,4]");
        let numbers_vec: Vec<u8> = vec![1u8, 2u8, 3u8, 4u8];
        assert_eq!(to_json_string(&numbers_vec), "[1,2,3,4]");
        let strings: [&str; 4] = ["a", "b", "c", "d"];
        assert_eq!(to_json_string(&strings[..]), "[\"a\",\"b\",\"c\",\"d\"]");

        // // Hash-maps:
        let mut map = std::collections::HashMap::<String, String>::new();
        map.insert("Hello".to_owned(), "World".to_owned());
        assert_eq!(to_json_string(&map), "{\"Hello\":\"World\"}");

        // Objects:
        let mut object_str = String::new();
        let mut object_writer = JsonObjectWriter::new(&mut object_str);

        // // Values
        object_writer.value("number", 42i32);
        object_writer.value("slice", &numbers[..]);

        // Nested arrays
        let mut nested_array = object_writer.array("array");
        nested_array.value(42u32);
        nested_array.value("?");
        nested_array.end();

        // Nested objects
        let nested_object = object_writer.object("object");
        nested_object.end();

        object_writer.end();
        assert_eq!(
            &object_str,
            "{\"number\":42,\"slice\":[1,2,3,4],\"array\":[42,\"?\"],\"object\":{}}"
        );
    }

    #[test]
    fn test_duplicate_keys() {
        let mut object_str = String::new();
        {
            let mut object_writer = JsonObjectWriter::new(&mut object_str);
            object_writer.value("number", 42i32);
            object_writer.value("number", 43i32);
        }
        // Duplicates are not checked, this is by design!
        assert_eq!(&object_str, "{\"number\":42,\"number\":43}");
    }

    #[test]
    fn test_flush() {
        // this could also be a file writer.
        let mut writer = Vec::<u8>::new();

        let mut buffer = String::new();
        let mut array = JsonArrayWriter::new(&mut buffer);
        for i in 1i32..=1000000i32 {
            array.value(i);
            if array.buffer_len() > 2000 {
                array.output_buffered_data(&mut writer).unwrap();
            }
        }
        array.end();
        std::io::Write::write_all(&mut writer, buffer.as_bytes()).unwrap();

        assert!(buffer.len() <= 4000, "Buffer too long");
        assert_eq!(
            &writer[writer.len() - b",999999,1000000]".len()..],
            b",999999,1000000]"
        );
    }

    #[test]
    fn test_encoding() {
        for c in 0x00..0x20 {
            let c = char::from(c);
            let json = to_json_string(c.to_string().as_str());
            assert!(&json[0..2] == "\"\\");
        }
        assert_eq!(
            to_json_string("</script >\0\x1F"),
            "\"<\\/script >\\u0000\\u001F\""
        );
    }

    #[test]
    fn test_pretty() {
        let mut res = String::new();
        let mut writer =
            JsonObjectWriter::with_formatter(&mut res, PrettyFormatter::with_indent("   "));
        {
            let mut nested_writer = writer.object("nested");
            nested_writer.value("a", 3);
            nested_writer.value("b", &vec![0, 1, 4]);
        }
        writer.value("c", &vec![true, false, true]);
        writer.value("d", NULL);
        // empty object
        writer.object("e");
        writer.array("f");
        writer.end();
        assert_eq!(
            res,
            r#"{
   "nested": {
      "a": 3,
      "b": [
         0,
         1,
         4
      ]
   },
   "c": [
      true,
      false,
      true
   ],
   "d": null,
   "e": {},
   "f": []
}"#
        );
    }
}
