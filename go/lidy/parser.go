package lidy

import (
	yaml "gopkg.in/yaml.v3"
)

// Builder is a user-implemented input-validation and creation of user objects
type Builder func(input Result) (interface{}, bool, error)

// Parser gathers the properties necessary to perform lidy parsing
type Parser struct {
	schema     YamlFile
	builderMap map[string]Builder
}

type tParserData struct {
	contentFileName string
	ruleSet         map[string]tRule
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
	parser := Parser{
		schema: YamlFile{
			File: file,
		},
		builderMap: builderMap,
	}
	yamlErr := parser.schema.Unmarshal()
	if yamlErr != nil {
		return parser, yamlErr
	}

	// METAPARSING VALIDATION
	// Validate that the provided schema is valid according to the lidy metaschema
	_, metaParsingError := metaParser.parseData(parser.schema)
	if metaParsingError != nil {
		return parser, metaParsingError
	}

	return parser, nil
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
		ruleSet:         map[string]tRule{},
		contentFileName: content.Name,
	}

	parserDocument := p.schema.Yaml.Content[0]

	for k := 0; k < len(parserDocument.Content); k += 2 {
		ruleName := parserDocument.Content[k].Value
		builder := p.builderMap[ruleName]
		parserData.ruleSet[ruleName] = tRule{
			name:           ruleName,
			node:           parserDocument.Content[k+1],
			builder:        builder,
			ruleIsMatching: map[*yaml.Node]bool{},
		}
	}

	contentDocument := content.Yaml.Content[0]

	return applyRule(parserData, "main", contentDocument)
}
