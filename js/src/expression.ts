import * as yaml from "yaml"
import { JoinError } from "./error"
import { applyInMatcher } from "./in"
import { ParserData } from "./lidy"
import { applyListMatcher } from "./list"
import { applyMapMatcher } from "./map"
import { applyOneOfMatcher } from "./oneOf"
import { applyRangeMatcher } from "./range"
import { applyRegexMatcher } from "./regex"
import { Result } from "./result"
import { applyRule } from "./rule"
import { applySizeCheck } from "./size"

export function applyExpression(
  parserData: ParserData,
  schema: yaml.YAMLMap<any, any>,
  content: yaml.Node,
): Result<any> {
  if (yaml.isScalar<any>(schema)) {
    const { value } = schema
    if (typeof value !== "string") {
      throw new Error(
        `encountered a value: ${value} (type: ${typeof value}) where an expression was expected`,
      )
    }
    return applyRule(parserData, value, content)
  }

  // Else, the expression must be a mapping
  if (!yaml.isMap<any, any>(schema)) {
    throw new Error(
      "Lidy expressions must be strings (rule names) or mappings (checkers)",
    )
  }

  let _map,
    _mapFacultative,
    _mapOf,
    _merge,
    _list,
    _listFacultative,
    _listOf,
    _min,
    _max,
    _nb

  for (const { key, value } of schema.items) {
    switch (key.value) {
      case "_regex":
        return applyRegexMatcher(parserData, value, content)
      case "_in":
        return applyInMatcher(parserData, value, content)
      case "_range":
        return applyRangeMatcher(parserData, value, content)
      case "_oneOf":
        return applyOneOfMatcher(parserData, value, content)
      case "_map":
        _map = value
        break
      case "_mapFacultative":
        _mapFacultative = value
        break
      case "_mapOf":
        _mapOf = value
        break
      case "_merge":
        _merge = value
        break
      case "_list":
        _list = value
        break
      case "_listFacultative":
        _listFacultative = value
        break
      case "_listOf":
        _listOf = value
        break
      case "_min":
        _min = value
        break
      case "_max":
        _max = value
        break
      case "_nb":
        _nb = value
        break
      default:
        throw new Error(`Unknown keyword found in matcher: '${key.value}'`)
    }
  }

  let joinError = new JoinError()
  if (_min || _max || _nb) {
    joinError.add(applySizeCheck(parserData, content as any, _min, _max, _nb))
  }
  let result: Result<any> | undefined
  if (_map || _mapFacultative || _mapOf || _merge) {
    try {
      result = applyMapMatcher(
        parserData,
        _map,
        _mapFacultative,
        _mapOf,
        _merge,
        content,
      )
    } catch (e) {
      joinError.add(e)
    }
  }
  if (_list || _listFacultative || _listOf) {
    try {
      result = applyListMatcher(
        parserData,
        _list,
        _listFacultative,
        _listOf,
        content,
      )
    } catch (e) {
      joinError.add(e)
    }
  }
  joinError.throw()
  if (result) {
    return result
  } else {
    throw new Error("No keyword found in matcher")
  }
}
