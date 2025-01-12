import * as yaml from "yaml"

import { File } from "./file"
import { checkRuleSet, makeMetaParserFor } from "./metaparser"
import { Result } from "./result"
import { applyRule, Rule } from "./rule"
import { PairMap } from "./util/pairMap"
import { createYamlFile, unmarshalYamlFile, YamlFile } from "./yamlfile"

// Builder is a user-implemented input-validation and creation of user objects
export type Builder = (input: Result<any>) => {
  data: any
  isLidyData: boolean
}
export interface Parser {
  ruleSet: Record<string, Rule>
  parse: (contentFile: File) => Result<any>
  _parseData: (content: YamlFile) => Result<any>
}

export interface ParserData {
  contentFileName: string
  parser: Parser
  ruleTrace: string[]
  // ruleIsMatchingNode is used to detect cycles in the rule set
  ruleIsMatchingNode: PairMap<string, yaml.Node, boolean>
  lineCounter: yaml.LineCounter
}

export function makeParser(
  file: File,
  builderMap: Record<string, Builder>,
): Parser {
  const schemaFile = createYamlFile(file)
  const error = unmarshalYamlFile(schemaFile)
  if (error) {
    throw error
  }

  const ruleSet = makeRuleSet(schemaFile, builderMap)

  const parser = makeParserFromRuleSet(ruleSet)

  // METAPARSING VALIDATION
  // Validate that the provided schema is valid according to the lidy metaschema
  makeMetaParserFor(parser)._parseData(schemaFile)

  checkRuleSet(ruleSet)

  return parser
}

export function makeRuleSet(
  yamlFile: YamlFile,
  builderMap: Record<string, Builder>,
): Record<string, Rule> {
  if (!yaml.isMap<any, any>(yamlFile.yaml)) {
    throw new Error("The document should be a YAML map.")
  }
  const ruleSet: Record<string, Rule> = {}

  yamlFile.yaml.items.forEach(({ key, value }) => {
    const ruleName: string = key.value
    const builder = builderMap[ruleName]
    ruleSet[ruleName] = {
      name: ruleName,
      node: value,
      builder,
      isUsed: false,
    }
  })

  return ruleSet
}

export function makeParserFromRuleSet(ruleSet: Record<string, Rule>): Parser {
  const parser = {
    ruleSet,
    parse: (contentFile: File) => {
      const yamlContentFile = createYamlFile(contentFile)
      const error = unmarshalYamlFile(yamlContentFile)
      if (error) {
        throw error
      }
      return parser._parseData(yamlContentFile)
    },
    _parseData: (content: YamlFile): Result<any> => {
      const parserData: ParserData = {
        parser,
        contentFileName: content.file.name,
        ruleTrace: [],
        ruleIsMatchingNode: new PairMap(),
        lineCounter: content.lineCounter,
      }

      return applyRule(parserData, "main", content.yaml)
    },
  }
  return parser
}
