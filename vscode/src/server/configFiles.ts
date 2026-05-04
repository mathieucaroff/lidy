import * as fs from "fs"
import * as path from "path"
import { minimatch } from "minimatch"
import * as yaml from "yaml"
import {
  Diagnostic,
  DiagnosticSeverity,
  Range,
} from "vscode-languageserver/node"
import {
  makeDiagnostic,
  nodeDiagnostic,
  yamlErrorToDiagnostic,
} from "./diagnostics"
import { fsPathToUri, toPosix } from "./paths"

export type ParsedAssociation = {
  pattern: string
  schema: string
}

export type ParsedConfigFile = {
  priority: number
  filePath: string
  matchingAssociation?: ParsedAssociation
}

type SendDiagnostics = (uri: string, diagnostics: Diagnostic[]) => void

export function readRepositoryConfigFiles(
  workspaceFolder: string,
  targetFilePath: string,
  sendDiagnostics: SendDiagnostics,
): ParsedConfigFile[] {
  const entries = fs.existsSync(workspaceFolder)
    ? fs.readdirSync(workspaceFolder, { withFileTypes: true })
    : []

  const result: ParsedConfigFile[] = []
  for (const entry of entries) {
    if (!entry.isFile()) {
      continue
    }
    const priority = configFilePriority(entry.name)
    if (priority === undefined) {
      continue
    }
    const filePath = path.join(workspaceFolder, entry.name)
    const diagnostics: Diagnostic[] = []
    const matchingAssociation = parseConfigFile(
      filePath,
      targetFilePath,
      workspaceFolder,
      diagnostics,
    )
    sendDiagnostics(fsPathToUri(filePath), diagnostics)
    result.push({ priority, filePath, matchingAssociation })
  }

  const configPath = path.join(workspaceFolder, "lidy.config.yaml")
  if (
    !result.some((entry) => entry.priority === -1) &&
    fs.existsSync(configPath)
  ) {
    const diagnostics: Diagnostic[] = []
    const matchingAssociation = parseConfigFile(
      configPath,
      targetFilePath,
      workspaceFolder,
      diagnostics,
    )
    sendDiagnostics(fsPathToUri(configPath), diagnostics)
    result.push({
      priority: -1,
      filePath: configPath,
      matchingAssociation,
    })
  }

  return result.sort((left, right) => left.priority - right.priority)
}

export function publishWorkspaceConfigDiagnostics(
  workspaceFolders: string[],
  openDocumentPaths: string[],
  sendDiagnostics: SendDiagnostics,
): void {
  for (const folder of workspaceFolders) {
    if (!fs.existsSync(folder)) {
      continue
    }
    const entries = fs
      .readdirSync(folder, { withFileTypes: true })
      .filter((entry) => entry.isFile())
    for (const entry of entries) {
      if (!looksLikeRepositoryConfigFileName(entry.name)) {
        continue
      }
      const filePath = path.join(folder, entry.name)
      if (isRepositoryConfigFile(filePath)) {
        const targetFilePath = openDocumentPaths[0] ?? filePath
        const diagnostics: Diagnostic[] = []
        parseConfigFile(filePath, targetFilePath, folder, diagnostics)
        sendDiagnostics(fsPathToUri(filePath), diagnostics)
      } else {
        sendDiagnostics(fsPathToUri(filePath), [
          makeDiagnostic(
            "Invalid Lidy configuration file name",
            DiagnosticSeverity.Warning,
            Range.create(0, 0, 0, 1),
          ),
        ])
      }
    }
  }
}

export function isRepositoryConfigFile(filePath: string): boolean {
  return configFilePriority(path.basename(filePath)) !== undefined
}

export function looksLikeInvalidRepositoryConfigFile(
  filePath: string,
): boolean {
  const fileName = path.basename(filePath)
  return (
    looksLikeRepositoryConfigFileName(fileName) &&
    !isRepositoryConfigFile(filePath)
  )
}

function parseConfigFile(
  configPath: string,
  targetFilePath: string,
  workspaceFolder: string,
  diagnostics: Diagnostic[],
): ParsedAssociation | undefined {
  const content = fs.readFileSync(configPath, "utf8")
  const lineCounter = new yaml.LineCounter()
  const document = yaml.parseDocument(content, { lineCounter })
  for (const error of document.errors) {
    diagnostics.push(yamlErrorToDiagnostic(error, lineCounter))
  }
  if (
    document.errors.length > 0 ||
    !document.contents ||
    !yaml.isMap(document.contents)
  ) {
    if (document.contents && !yaml.isMap(document.contents)) {
      diagnostics.push(
        makeDiagnostic(
          "Expected a YAML map",
          DiagnosticSeverity.Error,
          Range.create(0, 0, 0, 1),
        ),
      )
    }
    return undefined
  }

  const map = document.contents
  for (const item of map.items) {
    const key = scalarValue(item.key)
    if (key !== "associations") {
      diagnostics.push(
        nodeDiagnostic(
          item.key,
          lineCounter,
          `Unknown top-level key '${String(key)}'`,
          DiagnosticSeverity.Error,
        ),
      )
    }
  }
  const associationsNode = map.get("associations", true)
  if (!associationsNode) {
    return undefined
  }
  if (!yaml.isSeq(associationsNode)) {
    diagnostics.push(
      nodeDiagnostic(
        associationsNode,
        lineCounter,
        "'associations' must be a sequence",
        DiagnosticSeverity.Error,
      ),
    )
    return undefined
  }

  const relativePath = toPosix(path.relative(workspaceFolder, targetFilePath))
  let lastMatching: ParsedAssociation | undefined
  associationsNode.items.forEach((entry) => {
    if (!yaml.isMap(entry as yaml.Node)) {
      diagnostics.push(
        nodeDiagnostic(
          entry,
          lineCounter,
          "Association entry must be a map",
          DiagnosticSeverity.Error,
        ),
      )
      return
    }
    const entryMap = entry as yaml.YAMLMap<yaml.Node, yaml.Node>
    const patternNode = entryMap.get("pattern", true)
    const schemaNode = entryMap.get("schema", true)
    for (const item of entryMap.items) {
      const key = scalarValue(item.key)
      if (key !== "pattern" && key !== "schema") {
        diagnostics.push(
          nodeDiagnostic(
            item.key,
            lineCounter,
            `Unknown association key '${String(key)}'`,
            DiagnosticSeverity.Error,
          ),
        )
      }
    }
    if (!yaml.isScalar(patternNode) || typeof patternNode.value !== "string") {
      diagnostics.push(
        nodeDiagnostic(
          entry,
          lineCounter,
          "Association entry requires a string 'pattern'",
          DiagnosticSeverity.Error,
        ),
      )
      return
    }
    if (!yaml.isScalar(schemaNode) || typeof schemaNode.value !== "string") {
      diagnostics.push(
        nodeDiagnostic(
          entry,
          lineCounter,
          "Association entry requires a string 'schema'",
          DiagnosticSeverity.Error,
        ),
      )
      return
    }

    if (matchesPattern(relativePath, patternNode.value)) {
      lastMatching = { pattern: patternNode.value, schema: schemaNode.value }
    }
  })
  return lastMatching
}

function matchesPattern(relativePath: string, pattern: string): boolean {
  return minimatch(relativePath, pattern, {
    dot: true,
    nocase: false,
    nocomment: true,
  })
}

function scalarValue(node: unknown): unknown {
  return yaml.isScalar(node) ? node.value : undefined
}

function configFilePriority(fileName: string): number | undefined {
  if (fileName === "lidy.config.yaml") {
    return -1
  }
  const match = /^lidy\.override\.(0|[1-9][0-9]*)\.yaml$/.exec(fileName)
  if (!match) {
    return undefined
  }
  return Number(match[1])
}

function looksLikeRepositoryConfigFileName(fileName: string): boolean {
  return /^lidy\.(config|override\..+)\.ya?ml$/.test(fileName)
}
