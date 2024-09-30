# cairo

## Local Development
```bash
# Run the tests
cargo test

# Run the example server
cargo run --example server_final
```

## Attribution
This project uses software components that are licensed under the MIT License. While we do not use Axum directly, several components (particularly the `Router`, `PathRouter`, `Handlder`, `FromRequest`, and `FromRequestParts`) are heavily inspired by Axum. The following is the required attribution notice for the original work:

```
MIT License

Copyright (c) 2024 Tokio Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

This notice applies to the following components used in this project:

- Axum by Tokio Contributors (https://github.com/tokio-rs/axum)

By including this notice, we comply with the terms of the MIT License and ensure that proper credit is given to the original authors of the software components used in this project.
