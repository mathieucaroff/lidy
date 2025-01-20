import * as yaml from "yaml"

import { CheckError, CheckResultError, JoinError } from "./error"
import { readLocalFile } from "./file"
import { Builder, makeParserFromRuleSet, makeRuleSet, Parser } from "./lidy"
import { ListData, MapData, Position, Result } from "./result"
import { applyPredefinedRule, Rule } from "./rule"
import * as syaml from "./syaml"
import { deserializedYamlFile, YamlFile } from "./yamlfile"

export function makeMetaParserFor(subparser: Parser): Parser {
  const metaSchema: YamlFile = {
    file: readLocalFile("../../lidy.schema.yaml"),
    yaml: new yaml.YAMLMap(),
    doneParsing: false,
    lineCounter: new yaml.LineCounter(),
  }
  const error = deserializedYamlFile(metaSchema)
  if (error) {
    throw error
  }

  const checkMergedNode = (
    name: string,
    namePosition: Position,
    originPosition: Position,
  ): Error | undefined => {
    const rule = subparser.ruleSet[name]
    // checkError, to be returned only if the node is not a map checker
    const checkError = new CheckError(
      "_merge",
      `reference leads to a non-map-checker node`,
      metaParserData as any,
      rule.node,
    )

    if (!rule) {
      return new CheckResultError(
        "_merge",
        `unknown rule '${name}' at ${namePosition} encountered followings rules from a _merge keyword`,
        originPosition,
      )
    } else if (yaml.isScalar(rule.node)) {
      if (!syaml.isString(rule.node)) {
        return checkError
      }
      const { line, col: column } = metaParserData.lineCounter.linePos(
        rule.node.range?.[0] || 0,
      )
      const { line: lineEnd, col: columnEnd } =
        metaParserData.lineCounter.linePos(rule.node.range?.[1] || 0)
      const newNamePosition = {
        filename: originPosition.filename,
        line,
        column,
        lineEnd,
        columnEnd,
      }
      return checkMergedNode(rule.node.value, newNamePosition, originPosition)
    } else if (yaml.isMap<any, yaml.Node>(rule.node)) {
      const isMapChecker = rule.node.items.some(({ key }) => {
        if (
          key.value === "_map" ||
          key.value === "_mapFacultative" ||
          key.value === "_mapOf" ||
          key.value === "_merge"
        ) {
          return true
        }
      })
      if (!isMapChecker) {
        return checkError
      }
    }
  }

  const metaBuilderMap: Record<string, Builder> = {
    ruleReference: (input: Result<any>) => {
      const identifier = input.data
      if (identifier === "expression") {
        console.log("identifier === 'expression', stack:", new Error().stack)
      }
      const rule = subparser.ruleSet[identifier]
      if (!rule) {
        const $$onlyCheckIfRuleExistsRef = { value: false }
        applyPredefinedRule(
          {} as any,
          identifier,
          {} as any,
          $$onlyCheckIfRuleExistsRef,
        )

        // If the rule does not exist, we throw an error
        if (!$$onlyCheckIfRuleExistsRef.value) {
          const ruleListing = Object.keys(subparser.ruleSet).join(", ")
          throw new CheckResultError(
            identifier,
            `encountered unknown rule identifier '${identifier}'. Known rules are: [${ruleListing}]`,
            input.position,
          )
        }
        // Otherwise, nothing special to do, we can just return the input
      } else {
        rule.isUsed = true
      }
      return {
        data: input.data,
        isLidyData: true,
      }
    },
    mapChecker: (input: Result<any>) => {
      const mapData = input.data as MapData<any, string | ListData<any>>
      const _merge = mapData.map["_merge"]
      const joinError = new JoinError()
      if (_merge) {
        const mergedNodeSlice = (_merge.data as ListData<any>).listOf
        mergedNodeSlice.forEach((result) => {
          if (result.data.isLidyData && result.data._map) {
            // the merged node is a _map* checker, it's okay
            return // continue
          } else if (typeof result.data === "string") {
            // the merged node is an identifier, the rule it refers to.
            joinError.add(
              checkMergedNode(result.data, result.position, result.position),
            )
          }
        })
      }
      joinError.throw()
      return {
        data: input.data,
        isLidyData: true,
      }
    },
    sizedCheckerKeywordSet: (input: Result<MapData<any, any>>) => {
      const mapData = input.data
      ;["_min", "_max", "_nb"].forEach((keyword) => {
        const value = mapData.map[keyword]
        if (value && value.data < 0) {
          throw new CheckResultError(
            keyword,
            "cannot be negative",
            input.position,
          )
        }
      })
      const _min = mapData.map["_min"]
      const _max = mapData.map["_max"]
      const _nb = mapData.map["_nb"]
      if (_nb && _min) {
        throw new CheckResultError(
          "_nb",
          "it makes no sense to use the `_nb` and `_min` together",
          input.position,
        )
      }
      if (_nb && _max) {
        throw new CheckResultError(
          "_nb",
          "it makes no sense to use the `_nb` and `_max` together",
          input.position,
        )
      }
      if (_min && _max && _min.data > _max.data) {
        throw new CheckResultError(
          "_min",
          "`_max` cannot be lower than `_min`",
          input.position,
        )
      }
      return {
        data: input.data,
        isLidyData: true,
      }
    },
  }

  const metaRuleSet = makeRuleSet(metaSchema, metaBuilderMap)

  const metaParser = makeParserFromRuleSet(metaRuleSet)

  const metaParserData = {
    contentFileName: metaSchema.file.name,
    lineCounter: metaSchema.lineCounter,
    parser: metaParser,
    ruleTrace: [],
  }

  return metaParser
}

function checkDirectRuleReference(
  ruleSet: Record<string, Rule>,
  ruleNode: yaml.Node,
  ruleNameArray: string[],
): Error | undefined {
  if (yaml.isScalar(ruleNode)) {
    for (const ruleName of ruleNameArray) {
      if (ruleNode.value === ruleName) {
        return new Error(`rule '${ruleName}' references itself`)
      }
    }
    const targetRule = ruleSet[ruleNode.value as string]
    if (targetRule) {
      return checkDirectRuleReference(ruleSet, targetRule.node, [
        ...ruleNameArray,
        ruleNode.value as string,
      ])
    } else {
      // The rule is a predefined rule
      return undefined
    }
  } else if (!yaml.isMap<string, yaml.YAMLSeq<yaml.Node>>(ruleNode)) {
    throw new Error("rule node should be either a scalar or a mapping")
  }
  const directChildNodePair = ruleNode.items.find(({ key }) => {
    return (
      yaml.isScalar(key) && (key.value === "_oneOf" || key.value === "_merge")
    )
  })
  const joinError = new JoinError()
  if (directChildNodePair) {
    directChildNodePair.value!.items.forEach((node) => {
      joinError.add(checkDirectRuleReference(ruleSet, node, ruleNameArray))
    })
  }
  return joinError
}

export function checkRuleSet(ruleSet: Record<string, Rule>) {
  const joinError = new JoinError()
  const mainRule = ruleSet["main"]
  if (!mainRule) {
    joinError.add(new Error("could not find the 'main' rule"))
  } else {
    mainRule.isUsed = true
  }
  Object.entries(ruleSet).forEach(([name, rule]) => {
    if (!rule.isUsed) {
      joinError.add(new Error(`rule '${name}' is defined but never used`))
    }
  })
  Object.entries(ruleSet).forEach(([name, rule]) => {
    joinError.add(checkDirectRuleReference(ruleSet, rule.node, [name]))
  })
  joinError.throw()
}
