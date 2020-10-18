const path = require('path');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');
const webpack = require('webpack');
const WorkerPlugin = require('worker-plugin');

module.exports = {

  chainWebpack: (config) => {
    /* config.optimization
      .minimizer('terser')
      .tap((args) => {
        const { terserOptions } = args[0];
        console.log(args);
        terserOptions.keep_classnames = true;
        terserOptions.keep_fnames = true;
        return args;
      }); */

    config.module
      .rule('images')
      .use('url-loader')
      .tap((options) => ({ ...options, name: '[name].prout.[contenthash].[ext]' }));

    config.module
      .rule('images')
      .use('url-loader')
      .tap((options) => {
        // eslint-disable-next-line no-param-reassign
        options.fallback.options.name = 'img/[name].[contenthash].[ext]';
        // eslint-disable-next-line no-param-reassign
        options.name = '[name].[contenthash].[ext]';
        return options;
      });
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
  
  configureWebpack: {
    output: {
      chunkFilename: 'js/[name].[contenthash].js',
      filename: '[name].[contenthash].js',
    },
  },

  css: {
    extract: {
      filename: 'css/[name].[contenthash].css',
      chunkFilename: 'css/[name].[contenthash].css',
    },
  },
};
