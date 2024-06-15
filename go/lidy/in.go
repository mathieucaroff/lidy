package lidy

import yaml "gopkg.in/yaml.v3"

func applyInMatcher(parserData tParserData, node *yaml.Node, content *yaml.Node) (Result, error) {
	if content.Kind != yaml.ScalarNode {
		return Result{}, checkError("_in", "must be a scalar node", content)
	}

	for _, literalNode := range node.Content {
		if literalNode.Value == content.Value {
			return makeResult(parserData, content, content.Value), nil
		}
	}

	return Result{}, checkError("_in", "must be one of the accepted values", content)
}
