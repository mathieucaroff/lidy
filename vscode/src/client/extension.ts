import * as path from "path"
import * as vscode from "vscode"
import {
  CloseAction,
  ErrorAction,
  LanguageClient,
  LanguageClientOptions,
  RevealOutputChannelOn,
  ServerOptions,
  State,
  TransportKind,
} from "vscode-languageclient/node"
import { registerCommands } from "./commands"
import { refreshRemoteSchemasServerCommand } from "../shared/commandIds"

let client: LanguageClient | undefined

function formatState(state: State): string {
  switch (state) {
    case State.Starting:
      return "starting"
    case State.Running:
      return "running"
    case State.Stopped:
      return "stopped"
  }
}

export async function activate(
  context: vscode.ExtensionContext,
): Promise<void> {
  const serverModule = context.asAbsolutePath(
    path.join("out", "server", "main.js"),
  )
  const packagedSchemaRootPath = context.asAbsolutePath(
    path.join("out", "schema"),
  )

  const serverOptions: ServerOptions = {
    run: { module: serverModule, transport: TransportKind.stdio },
    debug: {
      module: serverModule,
      transport: TransportKind.stdio,
      options: { execArgv: ["--nolazy", "--inspect=6010"] },
    },
  }

  const configWatcher = vscode.workspace.createFileSystemWatcher(
    "**/lidy.config.yaml",
  )
  const overrideWatcher = vscode.workspace.createFileSystemWatcher(
    "**/lidy.override.*.yaml",
  )
  const outputChannel = vscode.window.createOutputChannel(
    "Lidy Language Server",
  )
  const traceOutputChannel = vscode.window.createOutputChannel(
    "Lidy Language Server Trace",
  )

  const clientOptions: LanguageClientOptions = {
    documentSelector: [
      { scheme: "file", language: "yaml" },
      { scheme: "untitled", language: "yaml" },
    ],
    outputChannel,
    traceOutputChannel,
    revealOutputChannelOn: RevealOutputChannelOn.Never,
    synchronize: {
      fileEvents: [configWatcher, overrideWatcher],
    },
    initializationOptions: {
      extensionRootPath: context.extensionPath,
      packagedSchemaRootPath,
      globalStoragePath: context.globalStorageUri.fsPath,
      isTrusted: vscode.workspace.isTrusted,
    },
    initializationFailedHandler: (error) => {
      outputChannel.appendLine(
        `[init-failed] ${error instanceof Error ? (error.stack ?? error.message) : String(error)}`,
      )
      return false
    },
    errorHandler: {
      error: (error, message, count) => {
        const messageLabel =
          message && typeof message === "object" && "method" in message
            ? String(message.method)
            : "<none>"
        outputChannel.appendLine(
          `[client-error] count=${String(count)} message=${messageLabel} error=${error.stack ?? error.message}`,
        )
        return { action: ErrorAction.Continue }
      },
      closed: () => {
        outputChannel.appendLine(
          "[client-closed] Language client connection closed",
        )
        return { action: CloseAction.DoNotRestart }
      },
    },
  }

  client = new LanguageClient(
    "lidy-language-server",
    "Lidy Language Server",
    serverOptions,
    clientOptions,
  )
  client.onDidChangeState((event) => {
    outputChannel.appendLine(
      `[state] ${formatState(event.oldState)} -> ${formatState(event.newState)}`,
    )
  })
  const clientStart = client.start()

  context.subscriptions.push(
    configWatcher,
    overrideWatcher,
    outputChannel,
    traceOutputChannel,
  )
  context.subscriptions.push(
    ...registerCommands(context, packagedSchemaRootPath, async () => {
      await clientStart
      if (!client) {
        throw new Error("Lidy language client is not available")
      }
      return client.sendRequest("workspace/executeCommand", {
        command: refreshRemoteSchemasServerCommand,
      })
    }),
  )
  outputChannel.appendLine(`[activate] Starting server module ${serverModule}`)
  await clientStart
  context.subscriptions.push(client)
}

export async function deactivate(): Promise<void> {
  if (client) {
    await client.stop()
    client = undefined
  }
}
