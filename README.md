# Protoview
Protoview is a util for exploring protobuf encoded bytes with and without a schema.

The project is structured into three workspace members:
- `protoview-lib`: Contains the decoding logic and pretty printing
- `protoview-cli`: Wraps the lib to be used in the terminal
- `protoview-gui`: Warps the lib in an easy to use GUI

## Installation
### CLI
To install the cli you can run the following command: `cargo install --git https://github.com/n3eo/protoview protoview-cli`

# TODO:
- protoview-lib
  - [ ] Varint field numbers in tag
  - [ ] Implement LEN variant
    - [x] Primitives 
    - [x] Sub messages
  - [ ] Float and double decoding in I32 & I64
  - [ ] Test bool and enum in Varint
  - [ ] Implement Mapping to schema
  - [ ] Extend test cases
  - [ ] Negative numbers (ZigZag)
  - [ ] Check proto docs if varint impl should use u128
  - [ ] Debug mode with duplicate fields for index?
    - [ ] Store parsed data in a tree structure
  - [ ] Schema parsing
- protoview-cli
  - [x] implement cli
  - [ ] pretty print
    - [ ] Fix recursive indentation
  - [ ] Add colors to output
  - [x] support bytes, hex
    - [x] auto mode
  - [x] stdin & file reading
- protoview-gui
  - [ ] Implement GUI
- TUI?
