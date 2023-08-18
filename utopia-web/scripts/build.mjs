import * as esbuild from 'esbuild'
import { wasmLoader } from 'esbuild-plugin-wasm'
import { buildOptions } from './common.mjs';

await esbuild.build(buildOptions);