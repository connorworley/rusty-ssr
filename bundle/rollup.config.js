import commonjs from '@rollup/plugin-commonjs';
import nodeResolve from '@rollup/plugin-node-resolve';
import replace from '@rollup/plugin-replace';
import sucrase from '@rollup/plugin-sucrase';

export default {
  input: 'src/index.js',
  plugins: [
    sucrase({
      exclude: ['node_modules/**'],
      transforms: ['jsx']
    }),
    commonjs(),
    nodeResolve({
      preferBuiltins: false,
    }),
    replace({
      'process.env.NODE_ENV': JSON.stringify('production'),
    }),
  ],
  output: {
    file: 'dist/main.js',
  },
};
