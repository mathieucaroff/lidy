import * as fs from "node:fs"
import { builtinModules } from "node:module"
import * as path from "node:path"
import dts from "unplugin-dts/vite"
import { defineConfig, type Plugin } from "vite"

const external = new Set([
  "yaml",
  ...builtinModules,
  ...builtinModules.map((name) => `node:${name}`),
])

function lidyMetaSchemaPlugin(): Plugin {
  const sourcePath = path.resolve(__dirname, "..", "lidy.schema.yaml")

  return {
    name: "lidy-meta-schema",
    apply: "build",
    buildStart() {
      this.addWatchFile(sourcePath)
    },
    generateBundle() {
      this.emitFile({
        type: "asset",
        fileName: "lidy.schema.yaml",
        source: fs.readFileSync(sourcePath, "utf8"),
      })
    },
  }
}

export default defineConfig({
  publicDir: false,
  build: {
    lib: {
      entry: path.resolve(__dirname, "src/index.ts"),
      formats: ["cjs"],
      fileName: () => "index.js",
    },
    outDir: "dist",
    emptyOutDir: true,
    minify: false,
    target: "node18",
    rollupOptions: {
      external: (id: string) => external.has(id),
    },
  },
  plugins: [
    lidyMetaSchemaPlugin(),
    dts({
      bundleTypes: true,
      insertTypesEntry: true,
      outDirs: ["dist"],
      tsconfigPath: "./tsconfig.json",
    }),
  ],
})
