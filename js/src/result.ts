import * as yaml from "yaml"
import { ParserData } from "./lidy"

export interface Position {
  filename: string
  line: number
  column: number
  lineEnd: number
  columnEnd: number
}

export interface Result<T> {
  position: Position
  ruleName: string
  isLidyData: boolean
  data: T
}

export interface MapData<TK, TV> {
  map: Record<string, Result<TV>>
  mapOf: KeyValueData<TK, TV>[]
}

export interface KeyValueData<S, T> {
  key: Result<S>
  value: Result<T>
}

export interface ListData<T> {
  list: Result<T>[]
  listOf: Result<T>[]
}

export function makeResult<T>(
  parserData: ParserData,
  content: yaml.Node,
  data: T,
): Result<T> {
  const { line, col: column } = parserData.lineCounter.linePos(
    content.range?.[0] || 0,
  )
  const { line: lineEnd, col: columnEnd } = parserData.lineCounter.linePos(
    content.range?.[1] || 0,
  )
  const [ruleName] = parserData.ruleTrace.slice(-1)

  return {
    position: {
      filename: parserData.contentFileName,
      line,
      column,
      lineEnd,
      columnEnd,
    },
    ruleName,
    isLidyData: true,
    data,
  }
}
