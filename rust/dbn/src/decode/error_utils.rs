pub fn silence_eof_error<T>(err: std::io::Error) -> std::io::Result<Option<T>> {
    if err.kind() == std::io::ErrorKind::UnexpectedEof {
        Ok(None)
    } else {
        Err(err)
    }
}
