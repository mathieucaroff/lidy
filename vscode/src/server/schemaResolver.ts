import * as crypto from "crypto"
import * as fs from "fs"
import * as path from "path"
import { minimatch } from "minimatch"
import {
  Diagnostic,
  DiagnosticSeverity,
  Range,
} from "vscode-languageserver/node"
import { makeDiagnostic } from "./diagnostics"
import { ParsedConfigFile } from "./configFiles"
import { findWorkspaceFolder, toPosix } from "./paths"
import { packagedSchemaPath, SchemaMetadata } from "../shared/schemaMetadata"

export type FileRef = { name: string; content: string }

type ResolveSchemaContext = {
  extensionRootPath: string
  globalStoragePath: string
  packagedSchemaRootPath: string
  readRepositoryConfigFiles: (
    workspaceFolder: string,
    targetFilePath: string,
  ) => ParsedConfigFile[]
  schemaMetadata: SchemaMetadata
  workspaceFolders: string[]
  workspaceTrusted: boolean
}

export async function resolveSchemaForDocument(
  filePath: string,
  content: string,
  context: ResolveSchemaContext,
): Promise<{ schemaFile?: FileRef; diagnostics: Diagnostic[] }> {
  const workspaceFolder = findWorkspaceFolder(
    filePath,
    context.workspaceFolders,
  )
  if (!workspaceFolder) {
    return { diagnostics: [] }
  }

  const configFiles = context.readRepositoryConfigFiles(
    workspaceFolder,
    filePath,
  )
  const overrideMatch = configFiles
    .filter((config) => config.priority >= 0 && config.matchingAssociation)
    .sort((left, right) => right.priority - left.priority)[0]
  if (overrideMatch?.matchingAssociation) {
    return resolveSchemaReference(
      overrideMatch.matchingAssociation.schema,
      overrideMatch.filePath,
      context,
    )
  }

  const directive = extractDirective(content)
  if (directive) {
    return resolveSchemaReference(directive, filePath, context)
  }

  const configMatch = configFiles.find(
    (config) => config.priority === -1,
  )?.matchingAssociation
  if (configMatch) {
    return resolveSchemaReference(
      configMatch.schema,
      path.join(workspaceFolder, "lidy.config.yaml"),
      context,
    )
  }

  const autodetected = autodetectSchema(
    filePath,
    workspaceFolder,
    context.schemaMetadata,
  )
  if (!autodetected) {
    return { diagnostics: [] }
  }
  return resolveSchemaReference(
    `schema/${autodetected}`,
    path.join(context.packagedSchemaRootPath, "metadata.json"),
    context,
  )
}

export async function clearRemoteCache(
  globalStoragePath: string,
): Promise<void> {
  const cacheDir = path.join(globalStoragePath, "remote-schema-cache")
  fs.rmSync(cacheDir, { recursive: true, force: true })
}

function extractDirective(content: string): string | undefined {
  const lines = content.split(/\r?\n/)
  let firstDirective: string | undefined
  for (const line of lines) {
    const trimmed = line.trim()
    if (trimmed.length === 0) {
      continue
    }
    if (!trimmed.startsWith("#")) {
      break
    }
    const match = /^#\s*lidy-schema:\s*(.+?)\s*$/.exec(trimmed)
    if (match && firstDirective === undefined) {
      firstDirective = match[1]
    }
  }
  return firstDirective
}

function autodetectSchema(
  filePath: string,
  workspaceFolder: string,
  schemaMetadata: SchemaMetadata,
): string | undefined {
  const relativePath = toPosix(path.relative(workspaceFolder, filePath))
  let candidate: string | undefined
  for (const entry of schemaMetadata.autodetectionByPath) {
    if (
      entry.pathList.some((pattern) => matchesPattern(relativePath, pattern))
    ) {
      candidate = entry.schema
    }
  }
  return candidate
}

async function resolveSchemaReference(
  schemaReference: string,
  originFilePath: string,
  context: ResolveSchemaContext,
): Promise<{ schemaFile?: FileRef; diagnostics: Diagnostic[] }> {
  try {
    if (schemaReference.startsWith("https://")) {
      const official = loadKnownOfficialSchema(schemaReference, context)
      if (official) {
        return { schemaFile: official, diagnostics: [] }
      }
      if (!context.workspaceTrusted) {
        return {
          diagnostics: [
            makeDiagnostic(
              "Remote schemas are disabled in restricted workspaces",
              DiagnosticSeverity.Error,
              Range.create(0, 0, 0, 1),
            ),
          ],
        }
      }
      return {
        schemaFile: await loadRemoteSchema(
          schemaReference,
          context.globalStoragePath,
        ),
        diagnostics: [],
      }
    }

    const resolvedPath = path.isAbsolute(schemaReference)
      ? schemaReference
      : schemaReference.startsWith("schema/")
        ? packagedSchemaPath(
            context.packagedSchemaRootPath,
            path.basename(schemaReference),
          )
        : path.resolve(path.dirname(originFilePath), schemaReference)
    return {
      schemaFile: {
        name: resolvedPath,
        content: fs.readFileSync(resolvedPath, "utf8"),
      },
      diagnostics: [],
    }
  } catch (error) {
    return {
      diagnostics: [
        {
          severity: DiagnosticSeverity.Error,
          message: error instanceof Error ? error.message : String(error),
          range: Range.create(0, 0, 0, 1),
          source: "lidy",
        },
      ],
    }
  }
}

function loadKnownOfficialSchema(
  url: string,
  context: ResolveSchemaContext,
): FileRef | undefined {
  const version = readExtensionVersion(context.extensionRootPath)
  const packagedFiles = context.schemaMetadata.autodetectionByPath.map(
    (entry) => entry.schema,
  )
  for (const schema of new Set(packagedFiles)) {
    const candidateUrls = [
      `https://raw.githubusercontent.com/mathieucaroff/lidy/refs/tags/v${version}/schema/${schema}`,
      `https://raw.githubusercontent.com/mathieucaroff/lidy/v${version}/schema/${schema}`,
    ]
    if (candidateUrls.includes(url)) {
      const schemaPath = packagedSchemaPath(
        context.packagedSchemaRootPath,
        schema,
      )
      return { name: schemaPath, content: fs.readFileSync(schemaPath, "utf8") }
    }
  }
  return undefined
}

async function loadRemoteSchema(
  url: string,
  globalStoragePath: string,
): Promise<FileRef> {
  const cacheFilePath = remoteCachePath(url, globalStoragePath)
  if (fs.existsSync(cacheFilePath)) {
    const stat = fs.statSync(cacheFilePath)
    if (Date.now() - stat.mtimeMs > 60 * 60 * 1000) {
      void fetchAndPersist(url, cacheFilePath)
    }
    return { name: url, content: fs.readFileSync(cacheFilePath, "utf8") }
  }

  const content = await fetchAndPersist(url, cacheFilePath)
  return { name: url, content }
}

async function fetchAndPersist(
  url: string,
  cacheFilePath: string,
): Promise<string> {
  const response = await fetch(url, { redirect: "follow" })
  if (!response.ok) {
    throw new Error(
      `Failed to fetch Lidy schema from ${url}: ${response.status} ${response.statusText}`,
    )
  }
  const content = await response.text()
  fs.mkdirSync(path.dirname(cacheFilePath), { recursive: true })
  fs.writeFileSync(cacheFilePath, content, "utf8")
  return content
}

function remoteCachePath(url: string, globalStoragePath: string): string {
  const key = crypto.createHash("sha256").update(url).digest("hex")
  return path.join(globalStoragePath, "remote-schema-cache", `${key}.yaml`)
}

function matchesPattern(relativePath: string, pattern: string): boolean {
  return minimatch(relativePath, pattern, {
    dot: true,
    nocase: false,
    nocomment: true,
  })
}

function readExtensionVersion(extensionRootPath: string): string {
  const packageJsonPath = path.join(extensionRootPath, "package.json")
  const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, "utf8")) as {
    version: string
  }
  return packageJson.version
}
