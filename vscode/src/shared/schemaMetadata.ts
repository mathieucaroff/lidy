import * as fs from "fs"
import * as path from "path"

export interface AutodetectionByPathEntry {
  schema: string
  pathList: string[]
}

export interface SchemaMetadata {
  autodetectionByPath: AutodetectionByPathEntry[]
}

export function loadSchemaMetadata(schemaRootPath: string): SchemaMetadata {
  const filePath = path.join(schemaRootPath, "metadata.json")
  const content = fs.readFileSync(filePath, "utf8")
  return JSON.parse(content) as SchemaMetadata
}

export function listPackagedSchemaFiles(schemaRootPath: string): string[] {
  const schemaDir = schemaRootPath
  return fs
    .readdirSync(schemaDir, { withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith(".schema.yaml"))
    .map((entry) => entry.name)
    .sort()
}

export function packagedSchemaPath(
  schemaRootPath: string,
  schemaFileName: string,
): string {
  return path.join(schemaRootPath, schemaFileName)
}
