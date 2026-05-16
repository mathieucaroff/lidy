import * as fs from "fs"

export interface AutodetectionByPathEntry {
  schema: string
  pathList: string[]
}

export interface SchemaMetadata {
  autodetectionByPath: AutodetectionByPathEntry[]
}

export function loadSchemaMetadata(
  schemaMetadataFilePath: string,
): SchemaMetadata {
  const content = fs.readFileSync(schemaMetadataFilePath, "utf8")
  return JSON.parse(content) as SchemaMetadata
}

export function listPackagedSchemaReferences(
  schemaMetadataFilePath: string,
): string[] {
  const schemaMetadata = loadSchemaMetadata(schemaMetadataFilePath)
  return [
    ...new Set(schemaMetadata.autodetectionByPath.map((entry) => entry.schema)),
  ].sort()
}
