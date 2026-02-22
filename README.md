# Protoview
Protoview is a util for exploring protobuf encoded bytes with and without a schema.

The project is structured into three workspace members:
- `protoview-lib`: Contains the decoding logic and pretty printing
- `protoview-cli`: Wraps the lib to be used in the terminal
- `protoview-gui`: Warps the lib in an easy to use GUI

# TODO:
- [ ] Varint field numbers in tag
- [ ] Implement LEN variant
- [ ] Float and double decoding in I32 & I64
- [ ] Test bool and enum in Varint
- [ ] Implement CLI
- [ ] Implement GUI
- [ ] Implement Mapping to schema
- [ ] Extend test cases