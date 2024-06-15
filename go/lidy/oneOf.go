package lidy

import (
	"fmt"

	yaml "gopkg.in/yaml.v3"
)

func applyOneOfMatcher(parserData tParserData, node *yaml.Node, content *yaml.Node) (Result, error) {
	// Starting to build errorSlice right away: it is used only if no match if found.
	errorDescription := fmt.Sprintf("none of the %d expressions matched", len(node.Content))
	errorSlice := []error{checkError("_oneOf", errorDescription, content)}

	for _, schema := range node.Content {
		result, err := applyExpression(parserData, schema, content)
		if err == nil {
			return result, nil
		}
		errorSlice = append(errorSlice, err)
	}
	return Result{}, joinError(errorSlice...)
}
