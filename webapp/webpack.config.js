const path = require("path");

module.exports = {
  entry: "./src/main.js",
  output: {
    path: path.resolve('../static/'),
    filename: 'bundled.js',
  },
  module: {
    rules: [
      {
        test: /\.js$/,
        exclude: /node_modules/,
        use: {
          loader: "babel-loader"
        }
      }
    ]
  }
};