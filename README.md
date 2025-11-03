# PNG Parser

A parser for PNG images that is written using Rust without any dependencies.

## Features

- Full PNG specification compliance: parses PNG data according to the official PNG 1.2 specification
- Chunk validation: ensures correct order, length, and integrity of all PNG chunks (IHDR, PLTE, IDAT, IEND, etc.)
- Field validation: checks for valid values in all critical fields (e.g., color type, bit depth, compression method)
- Pure Rust implementation: written entirely in safe Rust, with no external dependencies
- Custom error handling: provides clear, descriptive error messages for invalid or corrupted PNG data
- Extensible design: easily add support for additional PNG chunks or custom processing
- Performance-oriented: efficient parsing with minimal memory overhead
- Cross-platform: works on any platform supported by Rust
- Well-documented public API: all public fields and functions are documented for ease of use and understanding

## Resources

- [PNG Specification, Version 1.2 @ ligpng.org](http://www.libpng.org/pub/png/spec/1.2/PNG-Contents.html)

## License

This project is licensed under the terms of the MIT license. See the [LICENSE](LICENSE) file for details.
