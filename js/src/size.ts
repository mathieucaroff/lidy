import * as yaml from "yaml"
import { CheckError } from "./error"
import { ParserData } from "./lidy"

export function applySizeCheck(
  parserData: ParserData,
  content: yaml.YAMLMap<any, any> | yaml.YAMLSeq<any>,
  _min?: yaml.Scalar<number>,
  _max?: yaml.Scalar<number>,
  _nb?: yaml.Scalar<number>,
): Error | undefined {
  const size = content.items?.length
  if (size === undefined) {
    throw new Error("_(size), Only containers (maps or sequences) have a size.")
  }
  if (_min) {
    if (size < Number(_min.value)) {
      return new CheckError(
        "_min",
        `Expected container to have at least ${_min.value} entries but it has only ${size}.`,
        parserData,
        content,
      )
    }
  }
  if (_max) {
    if (size > Number(_max.value)) {
      return new CheckError(
        "_max",
        `Expected container to have at most ${_max.value} entries but it has ${size}.`,
        parserData,
        content,
      )
    }
  }
  if (_nb) {
    if (size !== Number(_nb.value)) {
      return new CheckError(
        "_nb",
        `Expected container to have exactly ${_nb.value} entries but it has ${size}.`,
        parserData,
        content,
      )
    }
  }
  return undefined
}
