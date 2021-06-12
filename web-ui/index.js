//The wasm application is compiled as javascript into the /pkg
//directory. Webpack then replaces this import with what is actually
//needed. To import the web assembly, the import FUNCTION must be
//used. The import STATEMENT does not work.
import './webui.scss';
import('./pkg');
