package lidy

import (
	"fmt"

	yaml "gopkg.in/yaml.v3"
)

type tMapInfo struct {
	MandatoryKeys map[string]bool
	Map           map[string]*yaml.Node
}

func resolveMergeReference(parserData tParserData, node *yaml.Node) *yaml.Node {
	if node.Kind == yaml.MappingNode {
		return node
	}
	if node.Kind != yaml.ScalarNode {
		panic("The merge values must be mappings or references to mappings")
	}
	rule, ruleFound := parserData.parser[node.Value]
	if !ruleFound {
		panic("The merge value reference must exist in the schema")
	}

	return resolveMergeReference(parserData, rule.node)
}

func contributeToMapInfo(parserData tParserData, mapInfoRef *tMapInfo, _map *yaml.Node, _mapFacultative *yaml.Node, _merge *yaml.Node) {
	if _merge != nil {
		for _, node := range _merge.Content {
			resolvedNode := resolveMergeReference(parserData, node)
			var mapNode, mapFacultativeNode, mergeNode *yaml.Node
			for k := 0; k < len(resolvedNode.Content); k += 2 {
				key := resolvedNode.Content[k]
				value := resolvedNode.Content[k+1]
				switch key.Value {
				case "_map":
					mapNode = value
				case "_mapFacultative":
					mapFacultativeNode = value
				case "_merge":
					mergeNode = value
				}
			}
			contributeToMapInfo(parserData, mapInfoRef, mapNode, mapFacultativeNode, mergeNode)
		}
	}
	if _map != nil {
		for k := 0; k < len(_map.Content); k += 2 {
			key := _map.Content[k]
			schema := _map.Content[k+1]
			mapInfoRef.Map[key.Value] = schema
			mapInfoRef.MandatoryKeys[key.Value] = true
		}
	}
	if _mapFacultative != nil {
		for k := 0; k < len(_mapFacultative.Content); k += 2 {
			key := _mapFacultative.Content[k]
			schema := _mapFacultative.Content[k+1]
			_, isMandatory := mapInfoRef.MandatoryKeys[key.Value]
			if !isMandatory {
				// We only update the map if the key is not mandatory:
				// A facultative key cannot override a mandatory one.
				mapInfoRef.Map[key.Value] = schema
			}
		}
	}
}

func applyMapMatcher(parserData tParserData, _map *yaml.Node, _mapFacultative *yaml.Node, _mapOf *yaml.Node, _merge *yaml.Node, content *yaml.Node) (Result, error) {
	if content.Kind != yaml.MappingNode {
		return Result{}, checkError("_map*", "must be a mapping node", content)
	}

	mapInfo := tMapInfo{
		MandatoryKeys: map[string]bool{},
		Map:           map[string]*yaml.Node{},
	}
	contributeToMapInfo(parserData, &mapInfo, _map, _mapFacultative, _merge)

	data := MapData{
		Map:   map[string]Result{},
		MapOf: []KeyValueData{},
	}
	errorSlice := []error{}

	mapContent := map[string]*yaml.Node{}
	for k := 0; k < len(content.Content); k += 2 {
		key := content.Content[k]
		value := content.Content[k+1]
		mapContent[key.Value] = value
	}

	for key := range mapInfo.MandatoryKeys {
		_, valueFound := mapContent[key]
		if !valueFound {
			err := checkError("_map", fmt.Sprintf("missing key '%s' in mapping", key), content)
			errorSlice = append(errorSlice, err)
		}
	}

	for k := 0; k < len(content.Content); k += 2 {
		key := content.Content[k]
		value := content.Content[k+1]
		unknownKey := true
		if key.Kind == yaml.ScalarNode {
			schema, schemaFound := mapInfo.Map[key.Value]
			if schemaFound {
				unknownKey = false
				result, err := applyExpression(parserData, schema, value)
				if err == nil {
					data.Map[key.Value] = result
				} else {
					errorSlice = append(errorSlice, fmt.Errorf("key %s: %w", key.Value, err))
				}
			}
		}
		if unknownKey {
			if _mapOf != nil {
				maybeErrorSlice := []error{
					checkError("_mapOf", fmt.Sprintf("none of the %d _mapOf associations matched", len(_mapOf.Content)/2), _mapOf),
				}
				matchFound := false
				for k := 0; k < len(_mapOf.Content); k += 2 {
					keyResult, keyErr := applyExpression(parserData, _mapOf.Content[k], key)
					valueResult, valueErr := applyExpression(parserData, _mapOf.Content[k+1], value)
					if keyErr == nil && valueErr == nil {
						data.MapOf = append(data.MapOf, KeyValueData{Key: keyResult, Value: valueResult})
						matchFound = true
						break
					} else {
						if keyErr != nil {
							maybeErrorSlice = append(maybeErrorSlice, checkError("_mapOf[key]", keyErr.Error(), key))
						}
						if valueErr != nil {
							maybeErrorSlice = append(maybeErrorSlice, checkError("_mapOf[value]", valueErr.Error(), value))
						}
					}
				}
				if !matchFound {
					errorSlice = append(errorSlice, joinError(maybeErrorSlice...))
				}
			} else {
				var err error
				if key.Kind != yaml.ScalarNode {
					err = checkError("_map*", "expected a scalar key in mapping", key)
				} else {
					err = checkError("_map*", fmt.Sprintf("unknown key '%s'", key.Value), key)
				}
				errorSlice = append(errorSlice, err)
			}
		}
	}

	return makeResult(parserData, content, data), joinError(errorSlice...)
}
