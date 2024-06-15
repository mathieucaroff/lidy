package lidy

import (
	yaml "gopkg.in/yaml.v3"
)

func applyListMatcher(parserData tParserData, _list *yaml.Node, _listFacultative *yaml.Node, _listOf *yaml.Node, content *yaml.Node) (Result, error) {
	if content.Kind != yaml.SequenceNode {
		return Result{}, checkError("_list*", "must be a sequence node", content)
	}

	data := ListData{}
	errorSlice := []error{}
	offset := 0

	if _list != nil {
		for k, schema := range _list.Content {
			if k >= len(content.Content) {
				errorSlice = append(errorSlice, checkError("_list", "not enough entries", content))
				break
			}
			result, err := applyExpression(parserData, schema, content.Content[k])
			errorSlice = append(errorSlice, err)
			data.List = append(data.List, result)
		}

		offset += len(_list.Content)
	}

	if _listFacultative != nil {
		for k, schema := range _listFacultative.Content {
			index := offset + k
			if index >= len(content.Content) {
				break
			}
			result, err := applyExpression(parserData, schema, content.Content[index])
			errorSlice = append(errorSlice, err)
			data.List = append(data.List, result)
		}

		offset += len(_listFacultative.Content)
	}

	if _listOf != nil {
		for k := offset; k < len(content.Content); k++ {
			result, err := applyExpression(parserData, _listOf, content.Content[k])
			errorSlice = append(errorSlice, err)
			data.ListOf = append(data.ListOf, result)
		}
	} else if offset < len(content.Content) {
		errorSlice = append(errorSlice, checkError("_list*", "too many entries", content))
	}

	return makeResult(parserData, content, data), joinError(errorSlice...)
}
