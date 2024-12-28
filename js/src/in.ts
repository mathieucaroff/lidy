import * as yaml from "yaml"
import { CheckError } from "./error"
import { ParserData } from "./lidy"
import { makeResult, Result } from "./result"

export function applyInMatcher(
  parserData: ParserData,
  node: yaml.YAMLMap<any, any>,
  content: yaml.Node,
): Result<string | number | boolean | null> {
  if (!yaml.isScalar<string | number | boolean | null>(content)) {
    throw new CheckError("_in", "must be a scalar node", parserData, content)
  }

  const acceptedValues = node.items.map(({ value }) => value)

  if (acceptedValues.includes(content.value)) {
    return makeResult(parserData, content, content.value)
  }

  throw new CheckError(
    "_in",
    `must be one of the accepted values (${acceptedValues.join(", ")}) but is ${
      content.value
    }`,
    parserData,
    content,
  )
}
