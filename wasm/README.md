# Web page deployment with Wasm binary

This is a WebAssembly build of Rustack.
See the language itself's [README](../README.md) for more details.

See the deployed web page here: https://msakuta.github.io/rustack/


## How to build

Run

```
$ mkdir dist && npm ci && npm run build
```

and now you have a web page in `dist/`.


## How to run development build

Run

```
$ npm start
```

It will build the wasm binary and start web browser to open the page.

## Credit

This app was made with template from wasm-pack [template][template-docs].

[template-docs]: https://rustwasm.github.io/docs/wasm-pack/tutorials/npm-browser-packages/index.html


## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.