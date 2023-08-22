import * as esbuild from 'esbuild'
import { buildOptions } from './common.mjs';

await esbuild.build(buildOptions);