import * as yaml from "yaml"
import { CheckError } from "./error"
import { ParserData } from "./lidy"
import { makeResult, Result } from "./result"

const rangeRegex =
  /(([0-9]+(\.[0-9]+)?) *(<=?) *)?(int|float)( *(<=?) *([0-9]+(\.[0-9]+)?))?/

export function applyRangeMatcher(
  parserData: ParserData,
  node: yaml.Scalar<string>,
  content: yaml.Node,
): Result<any> {
  if (!yaml.isScalar<number>(content) || typeof content.value !== "number") {
    throw new CheckError("_range", "must be a number", parserData, content)
  }

  const submatchArray = node.value.match(rangeRegex)
  if (!submatchArray) {
    throw new Error("Invalid range matcher")
  }
  const leftBoundary = Number(submatchArray[2])
  const leftOperator = submatchArray[4] ?? ""
  const numberType = submatchArray[5]
  const rightOperator = submatchArray[7] ?? ""
  const rightBoundary = Number(submatchArray[8])

  let errorDescription = "must be a number"
  if (numberType === "int") {
    errorDescription = "must be an integer"
  }
  let value = Number(content.value)
  if (
    Number.isNaN(value) ||
    (numberType === "int" && !Number.isInteger(value))
  ) {
    throw new CheckError("_range", errorDescription, parserData, content)
  }

  let ok = true
  ok =
    ok &&
    (leftOperator === "" ||
      (leftOperator === "<" && leftBoundary < value) ||
      (leftOperator === "<=" && leftBoundary <= value))
  ok =
    ok &&
    (rightOperator === "" ||
      (rightOperator === "<" && value < rightBoundary) ||
      (rightOperator === "<=" && value <= rightBoundary))

  if (!ok) {
    throw new CheckError(
      "_range",
      "must be inside the specified range",
      parserData,
      content,
    )
  }

  return makeResult(parserData, content, value)
}
