import typescript from '@rollup/plugin-typescript';
import resolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import builtins from 'rollup-plugin-node-builtins';
import json from '@rollup/plugin-json';
import dotenv from "rollup-plugin-dotenv"

export default {
  input: 'src/main.ts',
  output: [
    {
      file: 'dist/chessground-examples.js',
      format: 'iife',
      name: 'ChessgroundExamples',
    },
    // {
    //   file: 'dist/chessground-examples.min.js',
    //   format: 'iife',
    //   name: 'ChessgroundExamples',
    // },
  ],
  plugins: [
    resolve({
      exportConditions: ["default", "module", "import"],
      mainFields: ["browser", "module", "main"],
      modulesOnly: true,
      preferBuiltins: false
    }),
    typescript(), commonjs(), builtins(), json(), dotenv()
  ],
  onwarn: function (warning, handler) {
    // Ignore the warning for this in arrow functions written to undefined in dependencies.
    if (warning.code === 'THIS_IS_UNDEFINED') { return; }

    // console.warn everything else
    handler(warning);
  }
};
