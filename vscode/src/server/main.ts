import * as path from "path"
import { makeParser } from "lidy"
import {
  createConnection,
  Diagnostic,
  ExecuteCommandParams,
  InitializeParams,
  InitializeResult,
  ProposedFeatures,
  TextDocumentSyncKind,
} from "vscode-languageserver/node"
import { TextDocument } from "vscode-languageserver-textdocument"
import { TextDocuments } from "vscode-languageserver/node"
import { errorToDiagnostics } from "./diagnostics"
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
  globalStoragePath?: string
  isTrusted?: boolean
}

const connection = createConnection(
  ProposedFeatures.all,
  process.stdin,
  process.stdout,
)
const documents = new TextDocuments(TextDocument)

let extensionRootPath = ""
let packagedSchemaRootPath = ""
let globalStoragePath = ""
let workspaceTrusted = false
let workspaceFolders: string[] = []
let schemaMetadata: SchemaMetadata = { autodetectionByPath: [] }

connection.onInitialize((params: InitializeParams): InitializeResult => {
  const init = (params.initializationOptions ?? {}) as InitializationOptions
  extensionRootPath = init.extensionRootPath ?? ""
  packagedSchemaRootPath =
    init.packagedSchemaRootPath ?? path.join(extensionRootPath, "out", "schema")
  globalStoragePath =
    init.globalStoragePath ?? path.join(extensionRootPath, ".cache")
  workspaceTrusted = init.isTrusted === true
  workspaceFolders = (params.workspaceFolders ?? [])
    .map((folder) => uriToFsPath(folder.uri))
    .filter((folderPath): folderPath is string => folderPath !== undefined)
  schemaMetadata = loadSchemaMetadata(packagedSchemaRootPath)

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
  try {
    const parser = makeParser(schemaFile, {})
    parser.parse({ name: document.uri, content: document.getText() })
    return []
  } catch (error) {
    return errorToDiagnostics(document, error)
  }
}
