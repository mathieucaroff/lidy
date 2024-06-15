package lidy

import (
	"fmt"
	"regexp"
	"strings"
	"time"

	yaml "gopkg.in/yaml.v3"
)

const regexBase64Source = `^[a-zA-Z0-9_\- \n]*[= \n]*$`

var regexBase64 = regexp.MustCompile(regexBase64Source)

type tRule struct {
	name           string
	node           *yaml.Node
	builder        Builder
	ruleIsMatching map[*yaml.Node]bool
}

func applyRule(parserData tParserData, ruleName string, content *yaml.Node) (Result, error) {
	newParserData := tParserData{
		ruleSet:   parserData.ruleSet,
		ruleTrace: append(parserData.ruleTrace, ruleName),
	}

	rule, ruleFound := parserData.ruleSet[ruleName]
	if !ruleFound {
		return applyPredefinedRule(newParserData, ruleName, content)
	}
	_, ruleIsAlreadyProcsesingThisNode := rule.ruleIsMatching[content]
	if ruleIsAlreadyProcsesingThisNode {
		panic(fmt.Sprintf("Infinite loop: Rule %s encountered multiple times for the same node (%s %s)", ruleName, content.Tag, content.Value))
	}

	rule.ruleIsMatching[content] = true
	result, err := applyExpression(newParserData, rule.node, content)
	delete(rule.ruleIsMatching, content)

	if rule.builder != nil && err == nil {
		result.hasBeenBuilt = true
		data, isLidyData, buildErr := rule.builder(result)
		result.data = data
		result.isLidyData = isLidyData
		err = buildErr
	}

	if err != nil {
		text := err.Error()
		text = strings.ReplaceAll(text, "\n", "\n  ")
		err = fmt.Errorf("%s failed (\n%s\n)", ruleName, text)
	}

	return result, err
}

func applyPredefinedRule(parserData tParserData, ruleName string, content *yaml.Node) (Result, error) {
	var data interface{}
	errorText := ""
	switch ruleName {
	case "string":
		if content.Tag != "!!str" {
			errorText = "expected a string"
		}
		data = content.Value
	case "int":
		if content.Tag != "!!int" {
			errorText = "expected an integer"
		} else {
			var intData int
			err := content.Decode(&intData)
			if err != nil {
				return Result{}, err
			}
			data = intData
		}
	case "float":
		if content.Tag != "!!float" && content.Tag != "!!int" {
			errorText = "expected a float"
		} else {
			var floatData float64
			err := content.Decode(&floatData)
			if err != nil {
				return Result{}, err
			}
			data = floatData
		}
	case "binary":
		if content.Tag != "!!str" && content.Tag != "!!binary" {
			errorText = "expected a binary or string value"
		} else if !regexBase64.MatchString(content.Value) {
			errorText = fmt.Sprintf("expected a base64 value: a string which matches: /%s/", regexBase64Source)
		}
		data = content.Value
	case "boolean":
		if content.Tag != "!!bool" {
			errorText = "expected a boolean"
		} else {
			var boolData bool
			err := content.Decode(&boolData)
			if err != nil {
				return Result{}, err
			}
			data = boolData
		}
	case "nullType":
		if content.Tag != "!!null" {
			errorText = "expected the null value"
		}
	case "timestamp":
		if content.Tag != "!!str" && content.Tag != "!!timestamp" {
			errorText = "expected a timestamp (an ISO 8601 datetime)"
		} else {
			_, err := time.Parse(time.RFC3339Nano, content.Value)
			if err != nil {
				errorText = fmt.Sprintf("expected a timestamp (an ISO 8601 datetime; got error [%s])", err.Error())
			}
		}
		data = content.Value
	case "any":
		data = mapYamlToResultData(parserData, content)
	default:
		errorText = fmt.Sprintf("rule '%s' not found in the schema", ruleName)
	}
	if errorText != "" {
		return Result{}, checkError("", errorText, content)
	}
	return makeResult(parserData, content, data), nil
}

func mapYamlToResultData(parserData tParserData, content *yaml.Node) interface{} {
	switch content.Kind {
	case yaml.ScalarNode:
		return content.Value
	case yaml.MappingNode:
		data := MapData{}
		for i := 0; i < len(content.Content); i += 2 {
			key := content.Content[i]
			value := content.Content[i+1]
			keyResult := makeResult(parserData, key, mapYamlToResultData(parserData, key))
			valueResult := makeResult(parserData, value, mapYamlToResultData(parserData, value))
			data.MapOf = append(data.MapOf, KeyValueData{Key: keyResult, Value: valueResult})
		}
		return data
	case yaml.SequenceNode:
		data := ListData{}
		for _, value := range content.Content {
			data.ListOf = append(data.ListOf, makeResult(parserData, value, mapYamlToResultData(parserData, value)))
		}
		return data
	}
	return nil
}
