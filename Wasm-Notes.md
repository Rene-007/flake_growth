# Notes on Wasm support

Flake Growth can be excecuted within a browser by compiling it into wasm bytecode. For doing that it relies on the "stdweb" package. Unfortunately, the package is a bit deprecated by now, kiss3d does not support it anymore and there also seems to be some problems with rust compiler > 1.47. So, the wise desicion would be to switch to "web-sys", however, I didn't manage to get it running as smoothly as before. Therefore, here are the instruction to build it anyways:

1. The `Cargo.toml` should not exceed the following versions of kiss3d and nalgebra:
    ```
    ...
    [dependencies]
    kiss3d = "0.24"
    nalgebra = "0.21"
    ...
    ```

2. Install/set the rusc toolchain to maximum 1.47:
    ```
    $ rustup default 1.47
    ```

3. Download and install cargo-web:
    ```
    $ cargo install cargo-web
    ```

4. Compile *flake_growth* and automatically run it on a local server from where you can directly test it in your browser:
    ```
    $ cargo web start --release
    ...
        Finished release [optimized] target(s) in 0.38s
        Processing "flake_growth.wasm"...
        Finished processing of "flake_growth.wasm"!

    If you need to serve any extra files put them in the 'static' directory
    in the root of your crate; they will be served alongside your application.
    You can also put a 'static' directory in your 'src' directory.

    Your application is being served at '/flake_growth.js'. It will be automatically
    rebuilt if you make any changes in your code.

    You can access the web server at `http://[::1]:8000`.
    ```

5. Compile *flake_growth* into the wasm deploy folder:
    ```
    $ cargo web deploy --release
    ...
        Finished release [optimized] target(s) in 0.37s
        Processing "flake_growth.wasm"...
        Finished processing of "flake_growth.wasm"!
    The `flake_growth` was deployed to "./target/deploy/"
    ```
    and copy the files `index.html`, `flake_growth.js` and `flake_growth.wasm` onto your web server. You may also need a `.htaccess` file containing:
    ```
    AddType application/wasm wasm
    ```
    within the same folder.