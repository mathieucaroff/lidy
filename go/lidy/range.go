package lidy

import (
	"regexp"
	"strconv"

	yaml "gopkg.in/yaml.v3"
)

var rangeRegex = regexp.MustCompile(`(([0-9]+(\.[0-9]+)?) *(<=?) *)?(int|float)( *(<=?) *([0-9]+(\.[0-9]+)?))?`)

func applyRangeMatcher(parserData tParserData, node *yaml.Node, content *yaml.Node) (Result, error) {
	if content.Kind != yaml.ScalarNode || (content.Tag != "!!int" && content.Tag != "!!float") {
		return Result{}, checkError("_range", "must be a number", content)
	}

	submatchSlice := rangeRegex.FindStringSubmatch(node.Value)
	leftBoundary, _ := strconv.ParseFloat(submatchSlice[2], 64)
	leftOperator := submatchSlice[4]
	numberType := submatchSlice[5]
	rightOperator := submatchSlice[7]
	rightBoundary, _ := strconv.ParseFloat(submatchSlice[8], 64)

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

	ok := true

	ok = ok &&
		(leftOperator == "" ||
			leftOperator == "<" && leftBoundary < value ||
			leftOperator == "<=" && leftBoundary <= value)

	ok = ok &&
		(rightOperator == "" ||
			rightOperator == "<" && value < rightBoundary ||
			rightOperator == "<=" && value <= rightBoundary)

	if !ok {
		return Result{}, checkError("_range", "must be inside the specified range", content)
	}
	if numberType == "int" {
		return makeResult(parserData, content, intValue), nil
	}
	return makeResult(parserData, content, value), nil
}
