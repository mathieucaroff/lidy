import * as yaml from "yaml"
import { CheckError, JoinError } from "./error"
import { applyExpression } from "./expression"
import { ParserData } from "./lidy"
import { ListData, makeResult, Result } from "./result"

export function applyListMatcher(
  parserData: ParserData,
  _list: yaml.YAMLSeq<any>,
  _listFacultative: yaml.YAMLSeq<any>,
  _listOf: yaml.YAMLMap<any, any>,
  content: yaml.Node,
): Result<ListData<any>> {
  if (!yaml.isSeq<any>(content)) {
    throw new CheckError(
      "_list*",
      "must be a sequence node",
      parserData,
      content,
    )
  }

  const data: ListData<any> = {
    list: [],
    listOf: [],
  }
  const joinError = new JoinError()
  let offset = 0

  if (_list) {
    _list.items.some((schema, index) => {
      if (index >= content.items.length) {
        joinError.add(
          new CheckError("_list", "not enough entries", parserData, content),
        )
        return true
      }
      let result: Result<any> | undefined
      try {
        result = applyExpression(
          parserData,
          schema,
          content.items[index] as any,
        )
      } catch (e) {
        joinError.add(e)
      }
      if (result) {
        data.list.push(result)
      }
    })

    offset += _list.items.length
  }

  if (_listFacultative) {
    _listFacultative.items.some((schema, k) => {
      const index = offset + k
      if (index >= content.items.length) {
        return true
      }
      let result: Result<any> | undefined
      try {
        result = applyExpression(
          parserData,
          schema,
          content.items[index] as yaml.Node,
        )
      } catch (e) {
        joinError.add(e)
      }
      data.list.push(result)
    })

    offset += _listFacultative.items.length
  }

  if (_listOf) {
    for (let k = offset; k < content.items.length; k++) {
      let result: Result<any> | undefined
      try {
        result = applyExpression(
          parserData,
          _listOf,
          content.items[k] as yaml.Node,
        )
      } catch (e) {
        joinError.add(e)
      }
      if (result) {
        data.listOf.push(result)
      }
    }
  } else if (offset < content.items.length) {
    joinError.add(
      new CheckError("_list*", "too many entries", parserData, content),
    )
  }

  joinError.throw()

  return makeResult(parserData, content, data)
}
