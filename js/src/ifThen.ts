import * as yaml from "yaml"
import { CheckError, JoinError } from "./error"
import { applyExpression } from "./expression"
import { ParserData } from "./lidy"
import { Result } from "./result"

export function applyIfThenMatcher(
  parserData: ParserData,
  node: yaml.YAMLSeq<any>,
  content: yaml.Node,
): Result<any> {
  const joinError = new JoinError(
    new CheckError(
      "_ifThen",
      `none of the ${node.items.length} test expressions matched`,
      parserData,
      content,
    ),
  )

  for (const entry of node.items) {
    if (!yaml.isSeq<any>(entry) || entry.items.length !== 2) {
      throw new Error("_ifThen entries must be 2-item lists")
    }

    const [testExpression, thenExpression] = entry.items

    try {
      applyExpression(parserData, testExpression, content)
    } catch (error) {
      joinError.add(error as Error)
      continue
    }

    return applyExpression(parserData, thenExpression, content)
  }

  throw joinError
}
