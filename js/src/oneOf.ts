import * as yaml from "yaml"
import { CheckError, JoinError } from "./error"
import { applyExpression } from "./expression"
import { ParserData } from "./lidy"
import { Result } from "./result"

export function applyOneOfMatcher(
  parserData: ParserData,
  node: yaml.YAMLSeq<any>,
  content: yaml.Node,
): Result<any> {
  const joinError = new JoinError(
    new CheckError(
      "_oneOf",
      `none of the ${node.items.length} expressions matched`,
      parserData,
      content,
    ),
  )

  for (const schema of node.items) {
    try {
      return applyExpression(parserData, schema, content)
    } catch (error) {
      joinError.add(error)
    }
  }

  throw joinError
}
