# DataStruct

DataStruct is a Rust library that provides a set of data structures and utilities for working with binary data, JSON, and other formats. It includes a `Binary` struct for handling binary data, a `DValue` enum for representing various data types, and a parser for parsing data from strings.

## Features

- **Binary Data Handling**: The `Binary` struct allows you to work with binary data, including encoding and decoding to and from base64 strings.
- **Data Value Representation**: The `DValue` enum represents various data types, including strings, numbers, booleans, lists, dictionaries, and tuples.
- **JSON Serialization**: DataStruct provides JSON serialization and deserialization for the `DValue` enum.
- **Parser**: The library includes a parser for parsing data from strings into `DValue` instances.

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
datastruct-rs = "0.1.0"

```

## Binary Utility

```rust
use datastruct::binary_util::Binary;
use std::path::PathBuf;

// Create a new Binary instance from a vector of bytes
let binary = Binary::new(vec![72, 101, 108, 108, 111]);

// Read binary data from a file
let path = PathBuf::from("path/to/file");
let binary_from_file = Binary::from_file(path)?;

// Decode base64-encoded string to binary
let base64_string = "SGVsbG8gd29ybGQ=";
let binary_from_b64 = Binary::from_b64(base64_string.to_string())?;

```
## DValue Enum
```rust
use datastruct::DValue;
use std::collections::HashMap;

// Create different DValue instances
let string_value = DValue::String("Hello World".to_string());
let number_value = DValue::Number(42.0);
let boolean_value = DValue::Boolean(true);
let list_value = DValue::List(vec![DValue::Number(1.0), DValue::Number(2.0)]);
let dict_value = DValue::Dict(HashMap::new());

// Convert a DValue instance to JSON
let json_string = string_value.to_json();

// Parse a string to a DValue instance
let parsed_value = DValue::from("b:SGVsbG8gV29ybGQ=:");
let size = string_value.size();


```
## License
MIT License

