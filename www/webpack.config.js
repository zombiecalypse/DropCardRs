const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const CopyPlugin = require("copy-webpack-plugin");

module.exports = {
  entry: './www/index.js',
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: 'bootstrap.js',
  },
  plugins: [
    new HtmlWebpackPlugin({
      template: 'www/index.html'
    }),
    new CopyPlugin({
      patterns: [
        { from: "www/style.css", to: "style.css" },
      ],
    }),
  ],
  experiments: {
    asyncWebAssembly: true
  },
  mode: 'development'
};
