//The wasm application is compiled as javascript into the /pkg
//directory. Webpack then replaces this import with what is actually
//needed.
import("./pkg").then(runic => {
    console.log('module is: ', runic);
    runic.run_app();
});
