import { wasmLoader } from 'esbuild-plugin-wasm'

export const buildOptions = {
    entryPoints: ['src/index.tsx'],
    bundle: true,
    format: "esm",
    outfile: './public/utopia.mjs',
    plugins: [
        wasmLoader(),
    ],
};