package lidy

import (
	yaml "gopkg.in/yaml.v3"
)

// Builder is a user-implemented input-validation and creation of user objects
type Builder func(input Result) (interface{}, bool, error)

// Parser gathers the properties necessary to perform lidy parsing
type Parser map[string]tRule

type tParserData struct {
	contentFileName string
	parser          Parser
	ruleTrace       []string
}

// MustMakeParser makes a parser and panic if any error was found
func MustMakeParser(file File, builderMap map[string]Builder) Parser {
	parser, err := MakeParser(file, builderMap)
	if err != nil {
		panic(err)
	}
	return parser
}

// MakeParser tries to make a parser from the given file
func MakeParser(file File, builderMap map[string]Builder) (Parser, error) {
	schema := YamlFile{
		File: file,
	}
	yamlErr := schema.Unmarshal()
	if yamlErr != nil {
		return Parser{}, yamlErr
	}

	ruleSet := makeRuleSet(schema, builderMap)
	parser := Parser(ruleSet)

	// METAPARSING VALIDATION
	// Validate that the provided schema is valid according to the lidy metaschema
	_, metaParsingError := makeMetaParserFor(parser).parseData(schema)
	if metaParsingError != nil {
		return parser, metaParsingError
	}

	err := checkRuleSet(ruleSet)
	if err != nil {
		return parser, err
	}

	return parser, nil
}

func makeRuleSet(schema YamlFile, builderMap map[string]Builder) map[string]tRule {
	parserDocument := schema.Yaml.Content[0]
	ruleSet := map[string]tRule{}

	for k := 0; k < len(parserDocument.Content); k += 2 {
		ruleName := parserDocument.Content[k].Value
		builder := builderMap[ruleName]
		ruleSet[ruleName] = tRule{
			name:       ruleName,
			node:       parserDocument.Content[k+1],
			builder:    builder,
			isMatching: map[*yaml.Node]bool{},
		}
	}

	return ruleSet
}

// Parse -- use the parser to check the given YAML file, and produce a Lidy Result.
func (p Parser) Parse(content File) (Result, error) {
	contentData := YamlFile{
		File: content,
	}
	err := contentData.Unmarshal()
	if err != nil {
		return Result{}, err
	}
	return p.parseData(contentData)
}

func (p Parser) parseData(content YamlFile) (Result, error) {
	parserData := tParserData{
		parser:          p,
		contentFileName: content.Name,
	}

	contentDocument := content.Yaml.Content[0]

	return applyRule(parserData, "main", contentDocument)
}
