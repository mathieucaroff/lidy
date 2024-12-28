import * as yaml from "yaml"
import { toJS } from "yaml/util"

export function isString(node: yaml.Node): node is yaml.Scalar<string> {
  let result = yaml.isScalar(node) && typeof toJS(node) === "string"
  return result
}
