module.exports = {
    root: true,
    env: { browser: true, node: true },
    extends: [
        'eslint:recommended',
        'plugin:@typescript-eslint/recommended'
    ],
    parser: '@typescript-eslint/parser',
    parserOptions: { project: ['./tsconfig.json'], tsconfigRootDir: __dirname },
    plugins: [
        '@typescript-eslint'
    ]
};