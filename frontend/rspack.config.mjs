import path from 'node:path';
import { fileURLToPath } from 'node:url';
import rspack from '@rspack/core';

const dirname = path.dirname(fileURLToPath(import.meta.url));
const isDev = process.env.NODE_ENV !== 'production';

export default {
  infrastructureLogging: {
    level: 'error',
  },
  mode: isDev ? 'development' : 'production',
  target: ['web', 'es2024'],
  entry: {
    main: './src/main.ts',
  },
  devtool: isDev ? 'source-map' : false,
  output: {
    path: path.resolve(dirname, 'dist'),
    filename: 'assets/[name].[contenthash:8].js',
    publicPath: '/',
    clean: true,
  },
  resolve: {
    alias: {
      '@': path.resolve(dirname, 'src'),
    },
    extensions: ['.ts', '.js'],
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css',
        use: [
          {
            loader: 'postcss-loader',
            options: {
              postcssOptions: {
                plugins: {
                  '@tailwindcss/postcss': {},
                },
              },
            },
          },
        ],
      },
      {
        test: /\.ts$/,
        type: 'javascript/auto',
        use: [
          {
            loader: 'builtin:swc-loader',
            options: {
              jsc: {
                parser: {
                  syntax: 'typescript',
                  decorators: true,
                  explicitResourceManagement: true,
                },
                transform: {
                  decoratorVersion: '2022-03',
                  useDefineForClassFields: true,
                },
                target: 'es2024',
                experimental: {
                  runPluginFirst: true,
                  plugins: [
                    [
                      'swc-plugin-gem',
                      {
                        styleMinify: true,
                        // hmr: true,
                        selectorCompatible: true,
                        autoImport: {
                          extends: 'gem',
                          elements: {
                            '@': {
                              'ai-guard-(.*)': '/elements/$1',
                            },
                          },
                        },
                        autoImportDts: 'auto-import.d.ts',
                      },
                    ],
                  ],
                },
              },
            },
          },
        ],
      },
    ],
  },
  plugins: [
    new rspack.HtmlRspackPlugin({
      template: './public/index.html',
    }),
    new rspack.CopyRspackPlugin({
      patterns: [
        {
          from: path.resolve(dirname, 'public/icons'),
          to: 'icons',
        },
      ],
    }),
    new rspack.CssExtractRspackPlugin({
      filename: 'assets/[name].[contenthash:8].css',
    }),
  ],
  devServer: {
    host: '0.0.0.0',
    port: 5173,
    hot: true,
    historyApiFallback: true,
    proxy: [
      {
        context: ['/api'],
        target: 'http://127.0.0.1:8787',
        changeOrigin: true,
      },
    ],
  },
};
