package lidy

import (
	"fmt"
	"regexp"

	yaml "gopkg.in/yaml.v3"
)

func applyRegexMatcher(parserData tParserData, node *yaml.Node, content *yaml.Node) (Result, error) {
	regex := regexp.MustCompile(node.Value)
	if content.Kind != yaml.ScalarNode {
		return Result{}, checkError("_regex", "must be a scalar node", content)
	}
	if content.Tag != "!!str" {
		return Result{}, checkError("_regex", "must be a string", content)
	}
	if !regex.MatchString(content.Value) {
		return Result{}, checkError("_regex", fmt.Sprintf("must match regex /%s/", node.Value), content)
	}
	return makeResult(parserData, content, content.Value), nil
}
