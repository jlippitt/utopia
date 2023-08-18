import * as esbuild from 'esbuild'
import { buildOptions } from './common.mjs';

const ctx = await esbuild.context(buildOptions);

await ctx.watch();

let { host, port } = await ctx.serve({
    servedir: './public'
});

console.log(`Listening on ${host}:${port}`)
console.log('Watching for changes...');