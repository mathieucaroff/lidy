import * as fs from "fs"
import * as path from "path"
import * as vscode from "vscode"
import { listPackagedSchemaReferences } from "../shared/schemaMetadata"

type CommandHandler = () => Promise<void>

type ContributedCommand = {
  command: string
}

async function promptSchemaReference(
  packagedSchemaAutodetectionPath: string,
): Promise<string | undefined> {
  const packaged = listPackagedSchemaReferences(
    packagedSchemaAutodetectionPath,
  ).map((schemaReference) => ({
    label: path.basename(schemaReference),
    description: `Packaged official schema (${schemaReference})`,
    value: schemaReference,
  }))

  const entered = await vscode.window.showQuickPick(
    [
      ...packaged,
      {
        label: "$(edit) Enter custom schema reference",
        description: "Relative path, absolute path, or HTTPS URL",
        value: "",
      },
    ],
    { placeHolder: "Select a packaged schema or enter a custom reference" },
  )

  if (!entered) {
    return undefined
  }
  if (entered.value) {
    return entered.value
  }
  return vscode.window.showInputBox({
    prompt: "Schema reference",
    placeHolder: "./schema/custom.schema.yaml or https://...",
    ignoreFocusOut: true,
  })
}

function ensureConfigHeader(filePath: string): void {
  if (!fs.existsSync(filePath)) {
    fs.writeFileSync(filePath, "associations:\n", "utf8")
    return
  }
  const content = fs.readFileSync(filePath, "utf8")
  if (content.trim().length === 0) {
    fs.writeFileSync(filePath, "associations:\n", "utf8")
  }
}

function appendAssociation(
  filePath: string,
  pattern: string,
  schema: string,
): void {
  ensureConfigHeader(filePath)
  const content = fs.readFileSync(filePath, "utf8")
  const suffix = content.endsWith("\n") ? "" : "\n"
  const block = [
    `  - pattern: ${JSON.stringify(pattern)}`,
    `    schema: ${JSON.stringify(schema)}`,
  ].join("\n")
  fs.writeFileSync(filePath, `${content}${suffix}${block}\n`, "utf8")
}

function activeEditorOrThrow(): vscode.TextEditor {
  const editor = vscode.window.activeTextEditor
  if (!editor) {
    throw new Error("No active editor")
  }
  return editor
}

function workspaceFolderOrThrow(uri: vscode.Uri): vscode.WorkspaceFolder {
  const folder = vscode.workspace.getWorkspaceFolder(uri)
  if (!folder) {
    throw new Error("The active document is not inside a workspace folder")
  }
  return folder
}

function runCommand(
  callback: () => Thenable<void> | Promise<void>,
): CommandHandler {
  return async () => {
    try {
      await callback()
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      void vscode.window.showErrorMessage(message)
    }
  }
}

function declaredCommandIds(context: vscode.ExtensionContext): string[] {
  const packageJson = context.extension.packageJSON as {
    contributes?: { commands?: ContributedCommand[] }
  }
  return (packageJson.contributes?.commands ?? []).map((entry) => entry.command)
}

function ensureAllDeclaredCommandsRegistered(
  context: vscode.ExtensionContext,
  handlers: ReadonlyMap<string, CommandHandler>,
): void {
  const declared = declaredCommandIds(context)
  const missing = declared.filter((command) => !handlers.has(command))
  if (missing.length > 0) {
    throw new Error(
      `Missing handlers for contributed commands: ${missing.join(", ")}`,
    )
  }
}

export function registerCommands(
  context: vscode.ExtensionContext,
  packagedSchemaAutodetectionPath: string,
  refreshRemoteSchemas: () => Thenable<void>,
): vscode.Disposable[] {
  const handlers = new Map<string, CommandHandler>([
    [
      "lidy.insertSchemaDirective",
      runCommand(async () => {
        const editor = activeEditorOrThrow()
        const schema = await promptSchemaReference(
          packagedSchemaAutodetectionPath,
        )
        if (!schema) {
          return
        }

        const line = "# lidy-schema: " + schema
        await editor.edit((editBuilder) => {
          editBuilder.insert(new vscode.Position(0, 0), line + "\n")
        })
      }),
    ],
    [
      "lidy.addAssociationToConfig",
      runCommand(async () => {
        const editor = activeEditorOrThrow()
        const folder = workspaceFolderOrThrow(editor.document.uri)
        const pattern = await vscode.window.showInputBox({
          prompt: "Glob pattern",
          placeHolder: "deploy/**/*.yaml",
          ignoreFocusOut: true,
        })
        if (!pattern) {
          return
        }
        const schema = await promptSchemaReference(
          packagedSchemaAutodetectionPath,
        )
        if (!schema) {
          return
        }

        appendAssociation(
          path.join(folder.uri.fsPath, "lidy.config.yaml"),
          pattern,
          schema,
        )
        await refreshRemoteSchemas()
      }),
    ],
    [
      "lidy.addAssociationToOverride",
      runCommand(async () => {
        const editor = activeEditorOrThrow()
        const folder = workspaceFolderOrThrow(editor.document.uri)
        const level = await vscode.window.showInputBox({
          prompt: "Override priority",
          placeHolder: "0",
          ignoreFocusOut: true,
          validateInput: (value) =>
            /^(0|[1-9][0-9]*)$/.test(value)
              ? undefined
              : "Expected a non-negative integer without a leading zero",
        })
        if (level === undefined) {
          return
        }
        const pattern = await vscode.window.showInputBox({
          prompt: "Glob pattern",
          placeHolder: "deploy/**/*.yaml",
          ignoreFocusOut: true,
        })
        if (!pattern) {
          return
        }
        const schema = await promptSchemaReference(
          packagedSchemaAutodetectionPath,
        )
        if (!schema) {
          return
        }

        appendAssociation(
          path.join(folder.uri.fsPath, `lidy.override.${level}.yaml`),
          pattern,
          schema,
        )
        await refreshRemoteSchemas()
      }),
    ],
    [
      "lidy.refreshRemoteSchemas",
      runCommand(async () => {
        await refreshRemoteSchemas()
        vscode.window.setStatusBarMessage(
          "Lidy remote schema cache refreshed",
          3000,
        )
      }),
    ],
  ])

  ensureAllDeclaredCommandsRegistered(context, handlers)

  return [...handlers.entries()].map(([command, handler]) =>
    vscode.commands.registerCommand(command, handler),
  )
}
