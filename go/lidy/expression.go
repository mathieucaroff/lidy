package lidy

import (
	"fmt"

	yaml "gopkg.in/yaml.v3"
)

func applyExpression(parserData tParserData, schema *yaml.Node, content *yaml.Node) (Result, error) {
	// If the expression is a rule name, apply it
	if schema.Kind == yaml.ScalarNode {
		tag := schema.Tag
		value := schema.Value
		if tag != "!!str" {
			return Result{}, fmt.Errorf("encountered a value: '%s %s' where an expression was expected", tag, value)
		}
		return applyRule(parserData, value, content)
	}

	// Else, the expression must be a mapping
	if schema.Kind != yaml.MappingNode {
		panic("Lidy expressions must be strings (rule names) or mappings (checkers)")
	}

	var _map, _mapFacultative, _mapOf, _merge, _list, _listFacultative, _listOf, _min, _max, _nb *yaml.Node

	for k := 0; k < len(schema.Content); k += 2 {
		key := schema.Content[k]
		value := schema.Content[k+1]

		switch key.Value {
		case "_regex":
			return applyRegexMatcher(parserData, value, content)
		case "_in":
			return applyInMatcher(parserData, value, content)
		case "_range":
			return applyRangeMatcher(parserData, value, content)
		case "_oneOf":
			return applyOneOfMatcher(parserData, value, content)
		case "_map":
			_map = value
		case "_mapFacultative":
			_mapFacultative = value
		case "_mapOf":
			_mapOf = value
		case "_merge":
			_merge = value
		case "_list":
			_list = value
		case "_listFacultative":
			_listFacultative = value
		case "_listOf":
			_listOf = value
		case "_min":
			_min = value
		case "_max":
			_max = value
		case "_nb":
			_nb = value
		default:
			panic(fmt.Sprintf("Unknown keyword found in matcher: '%s'", key.Value))
		}
	}
	var sizeError error
	if _min != nil || _max != nil || _nb != nil {
		sizeError = applySizeCheck(_min, _max, _nb, content)
	}
	if _map != nil || _mapFacultative != nil || _mapOf != nil || _merge != nil {
		result, mapError := applyMapMatcher(parserData, _map, _mapFacultative, _mapOf, _merge, content)
		return result, joinError(mapError, sizeError)
	}
	if _list != nil || _listFacultative != nil || _listOf != nil {
		result, listError := applyListMatcher(parserData, _list, _listFacultative, _listOf, content)
		return result, joinError(listError, sizeError)
	}

	panic("No keyword found in matcher")
}
