import { builtinModules } from "node:module"
import * as path from "node:path"
import { defineConfig } from "vite"
import { viteStaticCopy } from "vite-plugin-static-copy"

const external = new Set([
  "vscode",
  "lidy",
  "minimatch",
  "yaml",
  "vscode-languageclient/node",
  "vscode-languageserver/node",
  "vscode-languageserver-textdocument",
  ...builtinModules,
  ...builtinModules.map((name) => `node:${name}`),
])

export default defineConfig({
  publicDir: false,
  build: {
    outDir: "out",
    emptyOutDir: true,
    sourcemap: true,
    target: "node18",
    minify: false,
    rollupOptions: {
      preserveEntrySignatures: "strict",
      input: {
        "client/extension": path.resolve(__dirname, "src/client/extension.ts"),
        "server/main": path.resolve(__dirname, "src/server/main.ts"),
      },
      output: {
        format: "cjs",
        entryFileNames: "[name].js",
      },
      external: (id) => external.has(id),
    },
  },
  plugins: [
    viteStaticCopy({
      targets: [
        {
          src: path.posix.join("..", "schema", "*.schema.yaml"),
          dest: "schema",
        },
        {
          src: path.posix.join("..", "schema", "metadata.json"),
          dest: "schema",
        },
      ],
    }),
  ],
})
