const path = require("path");
const { CleanWebpackPlugin } = require('clean-webpack-plugin');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const HtmlWebpackPlugin = require('html-webpack-plugin');

console.log('path:', path.resolve(__dirname, 'crate'));
module.exports = {
    experiments: {
        asyncWebAssembly: true
    },
    entry: './index.js',
    mode: "development",
    output: {
        path: path.resolve(__dirname, './dist'),
        filename: 'index_bundle.js',
    },

    plugins: [
        new CleanWebpackPlugin(),
        new HtmlWebpackPlugin(),
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, "crate"),
            outDir: path.resolve(__dirname, "pkg"),
            args: "--log-level warn",
            extraArgs: "--no-typescript",
        })
    ]
};
