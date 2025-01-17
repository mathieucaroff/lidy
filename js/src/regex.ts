import * as yaml from "yaml"
import { CheckError } from "./error"
import { ParserData } from "./lidy"
import { makeResult, Result } from "./result"

export function applyRegexMatcher(
  parserData: ParserData,
  node: yaml.Scalar<string>,
  content: yaml.Node,
): Result<any> {
  // Compile the regex scheam pattern
  const regex = new RegExp(node.value)

  // Check that the kind of the content node is a string
  if (!yaml.isScalar<string>(content)) {
    throw new CheckError("_regex", "must be a scalar node", parserData, content)
  }
  if (typeof content.value !== "string") {
    throw new CheckError("_regex", "must be a string", parserData, content)
  }
  if (!regex.test(content.value)) {
    throw new CheckError(
      "_regex",
      `must match regex /${node.value}/`,
      parserData,
      content,
    )
  }
  return makeResult(parserData, content, content.value)
}
