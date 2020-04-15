# wasm_raytracer
A raytracer shipped in WebASM

## Building
Requires  [`wasm-pack`](https://rustwasm.github.io/wasm-pack/) and rust installed.

Build with :
```
wasm-pack build --target web
```

## Veiwing
Start a static file server in the root directory e.g.
```
python3 -m http.server 8080 
```

Then navigate to `http://localhost:8080/` in a browser supporting WebASM and Modules
