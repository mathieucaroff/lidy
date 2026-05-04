import * as yaml from "yaml"
import {
  Diagnostic,
  DiagnosticSeverity,
  Range,
} from "vscode-languageserver/node"
import { TextDocument } from "vscode-languageserver-textdocument"

export function errorToDiagnostics(
  document: TextDocument,
  error: unknown,
): Diagnostic[] {
  const errors = flattenErrors(error)
  if (errors.length === 0) {
    return [
      makeDiagnostic(
        String(error),
        DiagnosticSeverity.Error,
        Range.create(0, 0, 0, 1),
      ),
    ]
  }

  return errors.map((entry) => {
    const match = /(\d+):(\d+)(?!.*\d+:\d+)/.exec(entry.message)
    if (!match) {
      return makeDiagnostic(
        entry.message,
        DiagnosticSeverity.Error,
        Range.create(0, 0, 0, 1),
      )
    }
    const line = Math.max(0, Number(match[1]) - 1)
    const character = Math.max(0, Number(match[2]) - 1)
    return makeDiagnostic(
      entry.message,
      DiagnosticSeverity.Error,
      Range.create(line, character, line, character + 1),
    )
  })
}

export function yamlErrorToDiagnostic(
  error: yaml.YAMLParseError,
  lineCounter: yaml.LineCounter,
): Diagnostic {
  const start = lineCounter.linePos(error.pos[0])
  const end = lineCounter.linePos(error.pos[1])
  return makeDiagnostic(
    error.message,
    DiagnosticSeverity.Error,
    Range.create(start.line - 1, start.col - 1, end.line - 1, end.col - 1),
  )
}

export function nodeDiagnostic(
  node: unknown,
  lineCounter: yaml.LineCounter,
  message: string,
  severity: DiagnosticSeverity,
): Diagnostic {
  const rangeNode = pairOrNodeToNode(node)
  const start = lineCounter.linePos(rangeNode.range?.[0] ?? 0)
  const end = lineCounter.linePos(rangeNode.range?.[1] ?? 1)
  return makeDiagnostic(
    message,
    severity,
    Range.create(
      start.line - 1,
      start.col - 1,
      end.line - 1,
      Math.max(start.col, end.col - 1),
    ),
  )
}

export function makeDiagnostic(
  message: string,
  severity: DiagnosticSeverity,
  range: Range,
): Diagnostic {
  return {
    message,
    severity,
    range,
    source: "lidy",
  }
}

function flattenErrors(error: unknown): Error[] {
  if (!(error instanceof Error)) {
    return []
  }
  const maybeJoin = error as Error & { errors?: unknown[] }
  if (Array.isArray(maybeJoin.errors)) {
    return maybeJoin.errors
      .flatMap((entry) => flattenErrors(entry))
      .filter((entry) => entry instanceof Error)
  }
  return [error]
}

function pairOrNodeToNode(node: unknown): yaml.Node {
  if (yaml.isNode(node)) {
    return node
  }
  if (node && typeof node === "object" && "key" in node) {
    return (node as yaml.Pair<yaml.Node, yaml.Node | null>).key
  }
  return new yaml.Scalar("")
}
