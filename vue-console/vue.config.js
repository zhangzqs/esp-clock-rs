const { defineConfig } = require("@vue/cli-service");
module.exports = defineConfig({
  transpileDependencies: true,
  devServer: {
    port: 8080,
    proxy: {
      "/posts": {
        target: "https://jsonplaceholder.typicode.com",
        changeOrigin: true,
        // pathRewrite: { "^/api": "" },
      },
    },
  },
});
