const path = require('path');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');
const webpack = require('webpack');
const WorkerPlugin = require('worker-plugin');

module.exports = {

  /* configureWebpack: (config) => {
    config.output.chunkFilename = 'js/[name].[hash:8].js';
  }, */

  chainWebpack: (config) => {
    // rust wasm bindgen https://github.com/rustwasm/wasm-bindgen
    config
      .plugin('worker-plugin')
      .use(WorkerPlugin)
      .end()
      .plugin('wasm-pack')
      .use(WasmPackPlugin)
      .init(
        (Plugin) => new Plugin({
          crateDirectory: path.resolve(__dirname, './rust'),
        }),
      )
      .end();
  },
  pluginOptions: {
    i18n: {
      locale: 'en',
      fallbackLocale: 'en',
      localeDir: 'locales',
      enableInSFC: true,
    },
  },
};
