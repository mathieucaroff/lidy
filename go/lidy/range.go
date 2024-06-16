package lidy

import (
	"regexp"
	"strconv"

	yaml "gopkg.in/yaml.v3"
)

var rangeRegex = regexp.MustCompile(`([0-9]+(\.[0-9]+)?) *(<=?) *(int|float) *(<=?) *([0-9]+(\.[0-9]+)?)`)

func applyRangeMatcher(parserData tParserData, node *yaml.Node, content *yaml.Node) (Result, error) {
	if content.Kind != yaml.ScalarNode || (content.Tag != "!!int" && content.Tag != "!!float") {
		return Result{}, checkError("_range", "must be a number", content)
	}

	submatchSlice := rangeRegex.FindStringSubmatch(node.Value)
	leftBoundary, _ := strconv.ParseFloat(submatchSlice[1], 64)
	leftOperator := submatchSlice[3]
	numberType := submatchSlice[4]
	rightOperator := submatchSlice[5]
	rightBoundary, _ := strconv.ParseFloat(submatchSlice[6], 64)

	var value float64
	var intValue int
	var err error
	if numberType == "int" {
		intValue, err = strconv.Atoi(content.Value)
		if err != nil {
			return Result{}, checkError("_range", "must be an integer", content)
		}
		value = float64(intValue)
	} else {
		value, err = strconv.ParseFloat(content.Value, 64)
		if err != nil {
			return Result{}, checkError("_range", "must be a number", content)
		}
	}

	if leftBoundary < value && value < rightBoundary ||
		leftOperator == "<=" && value == leftBoundary ||
		rightOperator == "<=" && value == rightBoundary {
		if numberType == "int" {
			return makeResult(parserData, content, intValue), nil
		}
		return makeResult(parserData, content, value), nil
	}

	return Result{}, checkError("_range", "must be inside the specified range", content)
}
