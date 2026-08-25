# dbn-c

C FFI bindings for the DBN crate, using [cbindgen](https://github.com/eqrion/cbindgen).

It supports:
- decoding DBN data with a push-based decoder for streaming or buffered input
- encoding DBN metadata
- CSV and JSON serialization of records

Records need no encoder: a DBN record is its wire representation, so writing one is
`fwrite(&record, record.length * 4, 1, file)`.

Compression is left to the caller.

## Generated header

The build writes `dbn.h` to `${target_directory}/include/dbn/dbn.h`, so consumers add
`${target_directory}/include` to their include path and use `#include <dbn/dbn.h>`.
Set `DBN_C_HEADER_DIR` to write the header to a specific directory instead.

## Release archives

Each DBN release attaches a `libdbn_c-<version>-<target>` archive per supported target, containing the static library, the shared library, the `dbn.h` header file, and `native-static-libs.txt`.
That last file holds the linker arguments the static library requires.

```sh
cc -o app app.c -Iinclude libdbn_c.a $(cat native-static-libs.txt)
```

The shared library resolves its own dependencies, so linking against it takes no extra arguments.

```sh
cc -o app app.c -Iinclude -L. -ldbn_c
```

## License

Distributed under the [Apache 2.0 License](https://www.apache.org/licenses/LICENSE-2.0.html).
