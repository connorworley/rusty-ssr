import alias from '@rollup/plugin-alias';
import commonjs from '@rollup/plugin-commonjs';
import inject from '@rollup/plugin-inject';
import nodeResolve from '@rollup/plugin-node-resolve';
import sucrase from '@rollup/plugin-sucrase';

export default {
  input: 'src/index.js',
  plugins: [
    sucrase({
      exclude: ['node_modules/**'],
      transforms: ['jsx']
    }),
    alias({
      entries: [
        { find: 'stream', replacement: 'stream-browserify' },
      ],
    }),
    commonjs(),
    nodeResolve({
      preferBuiltins: false,
    }),
    inject({
      process: 'process/browser',
    }),
  ],
  output: {
    file: 'dist/main.js',
  },
};
