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
        new HtmlWebpackPlugin({
            title: 'Tenebrous'
        }),
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, "crate"),
            outDir: path.resolve(__dirname, "pkg"),
            args: "--log-level warn",
            extraArgs: "--no-typescript",
        })
    ],
    module: {
        rules: [
            {
                test: /\.(scss)$/,
                use: [{
                    // inject CSS to page
                    loader: 'style-loader'
                }, {
                    // translates CSS into CommonJS modules
                    loader: 'css-loader'
                }, {
                    // Run postcss actions
                    loader: 'postcss-loader',
                    options: {
                        // `postcssOptions` is needed for postcss 8.x;
                        // if you use postcss 7.x skip the key
                        postcssOptions: {
                            // postcss plugins, can be exported to postcss.config.js
                            plugins: function () {
                                return [
                                    require('autoprefixer')
                                ];
                            }
                        }
                    }
                }, {
                    // compiles Sass to CSS
                    loader: 'sass-loader'
                }]
            }
        ]
    }
};
