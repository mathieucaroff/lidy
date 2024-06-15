package lidy

import (
	"fmt"

	yaml "gopkg.in/yaml.v3"
)

func makeMetaParserFor(parser Parser) Parser {
	metaSchema := YamlFile{
		File: ReadLocalFile("../../lidy.schema.yaml"),
	}
	err := metaSchema.Unmarshal()
	if err != nil {
		panic(err)
	}

	var checkMergedNode func(string) error
	checkMergedNode = func(name string) error {
		rule, ruleFound := parser[name]
		if !ruleFound {
			return fmt.Errorf("unknown rule '%s' encountered in _merge keyword", name)
		} else if rule.node.Kind == yaml.ScalarNode {
			if rule.node.Tag != "!!str" {
				return fmt.Errorf("encountered the non-string scalar '%s' where an identifier was expected", rule.node.Value)
			}
			return checkMergedNode(rule.node.Value)
		} else if rule.node.Kind == yaml.MappingNode {
			isMapChecker := false
			for k := 0; k < len(rule.node.Content); k += 2 {
				key := rule.node.Content[k].Value
				if key == "_map" || key == "_mapFacultative" || key == "_mapOf" || key == "_merge" {
					isMapChecker = true
					break
				}
			}
			if !isMapChecker {
				return checkError("_merge", "reference lead to a non-map-checker node", rule.node)
			}
		}
		return nil
	}

	metaBuilderMap := map[string]Builder{
		"identifier": func(input Result) (interface{}, bool, error) {
			var err error
			identifier := input.data.(string)
			_, ruleFound := parser[identifier]
			_, predefinedRuleFound := predefinedRuleNameSet[identifier]
			if !ruleFound && !predefinedRuleFound {
				err = fmt.Errorf("encountered unknown identifier '%s'", identifier)
			}
			return input.data, true, err
		},
		"mapChecker": func(input Result) (interface{}, bool, error) {
			mapData := input.data.(MapData)
			_, _mapFound := mapData.Map["_map"]
			_, _mapFacultativeFound := mapData.Map["_mapFacultative"]
			_, _mapOfFound := mapData.Map["_mapOf"]
			_merge, _mergeFound := mapData.Map["_merge"]
			if !_mapFound && !_mapFacultativeFound && !_mapOfFound && !_mergeFound {
				return nil, true, fmt.Errorf("expression maps must contain at least one mapChecker or listChecker keyword")
			}
			var errorSlice []error
			if _mergeFound {
				mergedNodeSlice := _merge.data.(ListData).ListOf
				for _, result := range mergedNodeSlice {
					if _, isMapData := result.data.(MapData); isMapData {
						continue
					} else if name, isString := result.data.(string); isString {
						errorSlice = append(errorSlice, checkMergedNode(name))
					} else {
						panic("_merge result data slice should contain only MapData for map checkers and strings for identifiers")
					}
				}
			}
			return input.data, true, joinError(errorSlice...)
		},
		"sizedCheckerKeywordSet": func(input Result) (interface{}, bool, error) {
			mapData := input.data.(MapData)
			_min, _minFound := mapData.Map["_min"]
			_max, _maxFound := mapData.Map["_max"]
			_, _nbFound := mapData.Map["_nb"]
			if _nbFound && _minFound {
				return nil, true, fmt.Errorf("it makes no sense to use the `_nb` and `_min` together")
			}
			if _nbFound && _maxFound {
				return nil, true, fmt.Errorf("it makes no sense to use the `_nb` and `_max` together")
			}
			if _minFound && _maxFound && _min.data.(int) > _max.data.(int) {
				return nil, true, fmt.Errorf("`_max` cannot be lower than `_min`")
			}
			return input.data, true, nil
		},
	}

	metaRuleSet := makeRuleSet(metaSchema, metaBuilderMap)

	return Parser(metaRuleSet)
}
