import * as yaml from "yaml"
import { CheckError } from "./error"
import { applyExpression } from "./expression"
import { Builder, ParserData } from "./lidy"
import { ListData, makeResult, MapData, Result } from "./result"

const regexBase64 = /^[a-zA-Z0-9_\- \n]*[= \n]*$/

export interface Rule {
  name: string
  node: yaml.YAMLMap<any, any>
  builder: Builder
  // isMatching is used to detect cycles in the rule set
  isMatching: Map<yaml.Node, boolean>
  // isUsed is used to detect unused rules
  isUsed: boolean
}

export function applyRule(
  parserData: ParserData,
  ruleName: string,
  content: yaml.Node,
): Result<any> {
  const newParserData = {
    ...parserData,
    ruleTrace: [...parserData.ruleTrace, ruleName],
  }

  const rule = parserData.parser.ruleSet[ruleName]
  if (!rule) {
    return applyPredefinedRule(newParserData, ruleName, content)
  }

  // Detect infinite loops while processing the data
  const ruleIsAlreadyProcessingThisNode = rule.isMatching.get(content)
  if (ruleIsAlreadyProcessingThisNode) {
    throw new CheckError(
      "_rule",
      `Infinite loop: Rule ${ruleName} encountered multiple times for the same node (${JSON.stringify(
        content,
      )})`,
      newParserData,
      content,
    )
  }

  let result: Result<any>
  try {
    try {
      rule.isMatching.set(content, true)
      result = applyExpression(newParserData, rule.node, content)
    } finally {
      rule.isMatching.delete(content)
    }

    if (rule.builder) {
      const { data, isLidyData } = rule.builder(result)
      result.data = data
      result.isLidyData = isLidyData
    }
  } catch (error) {
    let text = error.message
    text = text.replace(/\n/g, "\n  ")
    throw new Error(`${ruleName} failed (\n  ${text}\n)`)
  }

  return result
}

export function applyPredefinedRule(
  parserData: ParserData,
  ruleName: string,
  content: yaml.Node,
  $$onlyCheckIfRuleExistsRef?: { value: boolean },
): Result<any> {
  let predefinedRuleFunction = {
    string: () => {
      if (typeof value !== "string") {
        errorText = `expected a string`
      }
      data = value
    },
    int: () => {
      if (typeof value !== "number" || !Number.isInteger(value)) {
        errorText = `expected an integer`
      }
      data = value
    },
    float: () => {
      if (typeof value !== "number") {
        errorText = `expected a float`
      }
      data = value
    },
    binary: () => {
      if (typeof value !== "string") {
        errorText = `expected a binary or string value`
      } else if (!regexBase64.test(value)) {
        errorText = `expected a base64 value: a string which matches: /${regexBase64}/`
      }
      data = value
    },
    boolean: () => {
      if (typeof value !== "boolean") {
        errorText = `expected a boolean`
      }
      data = value
    },
    nullType: () => {
      if (value !== null) {
        errorText = `expected the null value`
      }
    },
    timestamp: () => {
      const errorString = `expected a timestamp (an ISO 8601 datetime)`
      if (typeof value !== "string") {
        errorText = errorString
      } else if (Number.isNaN(Number(new Date(value)))) {
        errorText = errorString
      }
      data = value
    },
    any: () => {
      data = null
    },
    anyData: () => {
      data = mapYamlToResultData(parserData, content)
    },
    never: () => {
      errorText = "encountered the never value"
    },
  }[ruleName]

  if ($$onlyCheckIfRuleExistsRef) {
    $$onlyCheckIfRuleExistsRef.value = predefinedRuleFunction !== undefined
    return
  }

  let data: any
  let errorText = ""
  let { value } = content as yaml.Scalar<any>
  predefinedRuleFunction
    ? predefinedRuleFunction()
    : (errorText = `rule '${ruleName}' not found in the schema`)

  if (errorText) {
    throw new CheckError("", errorText, parserData, content)
  }

  return makeResult(parserData, content, data)
}

function mapYamlToResultData(parserData: ParserData, content: yaml.Node): any {
  if (yaml.isScalar(content)) {
    return content.value
  } else if (yaml.isMap<yaml.Node, yaml.Node>(content)) {
    const data: MapData<any, any> = {
      map: {},
      mapOf: [],
    }
    content.items.forEach((item) => {
      data.mapOf.push({
        key: makeResult(
          parserData,
          item.key,
          mapYamlToResultData(parserData, item.key),
        ),
        value: makeResult(
          parserData,
          item.value!,
          mapYamlToResultData(parserData, item.value!),
        ),
      })
    })
    return data
  } else if (yaml.isSeq<yaml.Node>(content)) {
    const data: ListData<any> = {
      list: [],
      listOf: [],
    }
    content.items.forEach((item) => {
      data.listOf.push(
        makeResult(parserData, item, mapYamlToResultData(parserData, item)),
      )
    })
    return data
  }

  return null
}
