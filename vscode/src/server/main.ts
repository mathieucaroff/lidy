import * as path from "path"
import { Console } from "node:console"
import { makeParser } from "lidy"
import {
  createConnection,
  Diagnostic,
  DiagnosticSeverity,
  ExecuteCommandParams,
  InitializeParams,
  InitializeResult,
  ProposedFeatures,
  Range,
  TextDocumentSyncKind,
} from "vscode-languageserver/node"
import { TextDocument } from "vscode-languageserver-textdocument"
import { TextDocuments } from "vscode-languageserver/node"
import { errorToDiagnostics, makeDiagnostic } from "./diagnostics"
import {
  isRepositoryConfigFile,
  looksLikeInvalidRepositoryConfigFile,
  publishWorkspaceConfigDiagnostics,
  readRepositoryConfigFiles,
} from "./configFiles"
import { uriToFsPath } from "./paths"
import {
  clearRemoteCache,
  FileRef,
  resolveSchemaForDocument,
} from "./schemaResolver"
import { refreshRemoteSchemasServerCommand } from "../shared/commandIds"
import { loadSchemaMetadata, SchemaMetadata } from "../shared/schemaMetadata"

type InitializationOptions = {
  extensionRootPath?: string
  packagedSchemaRootPath?: string
  packagedSchemaAutodetectionPath?: string
  globalStoragePath?: string
  isTrusted?: boolean
}

const stderrConsole = new Console({
  stdout: process.stderr,
  stderr: process.stderr,
})

// The LSP protocol owns stdout. Redirect incidental library logging away from it.
console.log = stderrConsole.log.bind(stderrConsole)
console.info = stderrConsole.info.bind(stderrConsole)
console.debug = stderrConsole.debug.bind(stderrConsole)

const connection = createConnection(
  ProposedFeatures.all,
  process.stdin,
  process.stdout,
)
const documents = new TextDocuments(TextDocument)

let extensionRootPath = ""
let packagedSchemaRootPath = ""
let packagedSchemaAutodetectionPath = ""
let globalStoragePath = ""
let workspaceTrusted = false
let workspaceFolders: string[] = []
let schemaMetadata: SchemaMetadata = { autodetectionByPath: [] }

connection.onInitialize((params: InitializeParams): InitializeResult => {
  const init = (params.initializationOptions ?? {}) as InitializationOptions
  extensionRootPath = init.extensionRootPath ?? ""
  packagedSchemaRootPath =
    init.packagedSchemaRootPath ?? path.join(extensionRootPath, "out", "schema")
  packagedSchemaAutodetectionPath =
    init.packagedSchemaAutodetectionPath ??
    path.join(extensionRootPath, "out", "schema-autodetection.json")
  globalStoragePath =
    init.globalStoragePath ?? path.join(extensionRootPath, ".cache")
  workspaceTrusted = init.isTrusted === true
  workspaceFolders = (params.workspaceFolders ?? [])
    .map((folder) => uriToFsPath(folder.uri))
    .filter((folderPath): folderPath is string => folderPath !== undefined)
  schemaMetadata = loadSchemaMetadata(packagedSchemaAutodetectionPath)

  return {
    capabilities: {
      textDocumentSync: TextDocumentSyncKind.Incremental,
      executeCommandProvider: {
        commands: [refreshRemoteSchemasServerCommand],
      },
    },
  }
})

connection.onInitialized(() => {
  publishAllWorkspaceConfigDiagnostics()
})

documents.onDidOpen((event) => {
  void validateDocument(event.document)
})

documents.onDidChangeContent((event) => {
  void validateDocument(event.document)
})

documents.onDidClose((event) => {
  connection.sendDiagnostics({ uri: event.document.uri, diagnostics: [] })
})

connection.onDidChangeWatchedFiles(() => {
  void revalidateAll()
})

connection.onExecuteCommand(async (params: ExecuteCommandParams) => {
  if (params.command !== refreshRemoteSchemasServerCommand) {
    return
  }
  await clearRemoteCache(globalStoragePath)
  await revalidateAll()
})

documents.listen(connection)
connection.listen()

async function revalidateAll(): Promise<void> {
  publishAllWorkspaceConfigDiagnostics()
  for (const document of documents.all()) {
    await validateDocument(document)
  }
}

async function validateDocument(document: TextDocument): Promise<void> {
  const filePath = uriToFsPath(document.uri)
  if (!filePath) {
    return
  }

  if (
    isRepositoryConfigFile(filePath) ||
    looksLikeInvalidRepositoryConfigFile(filePath)
  ) {
    publishAllWorkspaceConfigDiagnostics()
    return
  }

  const resolution = await resolveSchemaForDocument(
    filePath,
    document.getText(),
    {
      extensionRootPath,
      globalStoragePath,
      packagedSchemaRootPath,
      packagedSchemaAutodetectionPath,
      readRepositoryConfigFiles: (workspaceFolder, targetFilePath) =>
        readRepositoryConfigFiles(
          workspaceFolder,
          targetFilePath,
          sendDiagnostics,
        ),
      schemaMetadata,
      workspaceFolders,
      workspaceTrusted,
    },
  )
  if (resolution.diagnostics.length > 0) {
    connection.sendDiagnostics({
      uri: document.uri,
      diagnostics: resolution.diagnostics,
    })
    return
  }
  if (!resolution.schemaFile) {
    connection.sendDiagnostics({ uri: document.uri, diagnostics: [] })
    return
  }

  const diagnostics = await lintYamlDocument(document, resolution.schemaFile)
  connection.sendDiagnostics({ uri: document.uri, diagnostics })
}

function publishAllWorkspaceConfigDiagnostics(): void {
  publishWorkspaceConfigDiagnostics(
    workspaceFolders,
    documents
      .all()
      .map((document) => uriToFsPath(document.uri))
      .filter((filePath): filePath is string => filePath !== undefined),
    sendDiagnostics,
  )
}

function sendDiagnostics(uri: string, diagnostics: Diagnostic[]): void {
  connection.sendDiagnostics({ uri, diagnostics })
}

async function lintYamlDocument(
  document: TextDocument,
  schemaFile: FileRef,
): Promise<Diagnostic[]> {
  let parser: ReturnType<typeof makeParser>
  try {
    parser = makeParser(schemaFile, {})
  } catch (error) {
    return schemaConstructionDiagnostics(schemaFile, error)
  }

  try {
    parser.parse({ name: document.uri, content: document.getText() })
    return []
  } catch (error) {
    return errorToDiagnostics(document, error)
  }
}

function schemaConstructionDiagnostics(
  schemaFile: FileRef,
  error: unknown,
): Diagnostic[] {
  const details = error instanceof Error ? error.message : String(error)
  return [
    makeDiagnostic(
      `Failed to construct parser from schema '${schemaFile.name}': ${details}`,
      DiagnosticSeverity.Error,
      Range.create(0, 0, 0, 1),
    ),
  ]
}
