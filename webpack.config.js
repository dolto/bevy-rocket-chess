const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

const dist = path.resolve(__dirname, "dist");

module.exports = {
  mode: "production",
  devtool: 'source-map',
  entry: {
    index: "./js/index.js"
  },
  output: {
    path: dist,
    filename: "[name].js"
  },
  experiments: {
    asyncWebAssembly: true, // 또는 syncWebAssembly: true
  },
  devServer: {
    static: {
      directory: dist
    },
    compress: true,
    port: 8080,
  },
  performance: {
    hints: false,
    maxEntrypointSize: 512000,
    maxAssetSize: 512000
  },
  plugins: [
    new CopyPlugin({
      patterns: [
        { from: path.resolve(__dirname, "static"), to: dist } // Modify the 'from' and 'to' paths accordingly
      ],
    }),

    new WasmPackPlugin({
      crateDirectory: __dirname,
    }),
  ]
};
