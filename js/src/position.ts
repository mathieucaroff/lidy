export interface Position {
  filename: string
  line: number
  column: number
  lineEnd: number
  columnEnd: number
}

export function simplePositionRepr(position: Position): string {
  return `${position.filename}:${position.line}:${position.column}`
}

export function shortPositionRepr(position: Position): string {
  return `${position.line}:${position.column}`
}
