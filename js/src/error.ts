import * as yaml from "yaml"
import { ParserData } from "./lidy"
import { Position } from "./result"

export class CheckError extends Error {
  constructor(
    keyword: string,
    description: string,
    parserData: ParserData,
    node: yaml.Node,
  ) {
    const { line, col } = parserData.lineCounter.linePos(node.range?.[0] || 0)
    super(`${keyword}: ${description} ${line}:${col}`)
  }
}

export class CheckResultError extends Error {
  constructor(ruleName: string, description: string, position: Position) {
    const { line, column } = position
    super(`${ruleName}: ${description} ${line}:${column}`)
  }
}

export class JoinError extends Error {
  errors: Error[]
  constructor(...errors: (Error | undefined | null)[]) {
    const err = ([] as Error[]).concat(
      ...errors.map((e) => {
        if (e instanceof JoinError) {
          return e.errors
        } else if (e) {
          return [e]
        } else {
          return []
        }
      }),
    )
    super()
    this.errors = err
    this.setMessage()
  }
  setMessage() {
    this.message = this.errors.map((e) => e?.message).join("; ")
  }
  add(error: Error | undefined) {
    if (error instanceof JoinError) {
      this.errors.push(...error.errors)
    } else if (error) {
      this.errors.push(error)
    }
    this.setMessage()
  }
  throw() {
    if (this.errors.length > 0) {
      throw this
    }
  }
}
