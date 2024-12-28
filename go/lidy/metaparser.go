package lidy

import (
	"fmt"

	yaml "gopkg.in/yaml.v3"
)

func makeMetaParserFor(subparser Parser) Parser {
	metaSchema := YamlFile{
		File: ReadLocalFile("../../lidy.schema.yaml"),
	}
	err := metaSchema.Unmarshal()
	if err != nil {
		panic(err)
	}

	var checkMergedNode func(string) error
	checkMergedNode = func(name string) error {
		rule, ruleFound := subparser[name]
		if !ruleFound {
			return checkError("_merge", fmt.Sprintf("unknown rule '%s' encountered in _merge keyword", name), rule.node)
		} else if rule.node.Kind == yaml.ScalarNode {
			if rule.node.Tag != "!!str" {
				return checkError("_merge", fmt.Sprintf("encountered the non-string scalar '%s' where an identifier was expected", rule.node.Value), rule.node)
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
				return checkError("_merge", "reference leads to a non-map-checker node", rule.node)
			}
		}
		return nil
	}

	metaBuilderMap := map[string]Builder{
		"ruleReference": func(input Result) (interface{}, bool, error) {
			var err error
			identifier := input.data.(string)
			rule, ruleFound := subparser[identifier]
			if !ruleFound {
				err = fmt.Errorf("encountered unknown rule identifier '%s'", identifier)
			}
			if ruleFound {
				rule.isUsed = true
				subparser[identifier] = rule
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

func checkDirectRuleReference(ruleSet map[string]tRule, ruleNode *yaml.Node, ruleNameSlice []string) error {
	if ruleNode.Kind == yaml.ScalarNode {
		for _, ruleName := range ruleNameSlice {
			if ruleNode.Value == ruleName {
				return fmt.Errorf("rule '%s' references itself", ruleNode.Value)
			}
		}
		if targetRule, targetRuleFound := ruleSet[ruleNode.Value]; targetRuleFound {
			return checkDirectRuleReference(
				ruleSet,
				targetRule.node,
				append(ruleNameSlice, ruleNode.Value),
			)
		} else {
			// The rule is a predefined rule
			return nil
		}
	} else if ruleNode.Kind != yaml.MappingNode {
		panic("rule node should be either a scalar or a mapping")
	}
	var directChildNode *yaml.Node
	for k := 0; k < len(ruleNode.Content); k += 2 {
		key := ruleNode.Content[k]
		if key.Kind == yaml.ScalarNode && (key.Value == "_oneOf" || key.Value == "_merge") {
			directChildNode = ruleNode.Content[k+1]
			break
		}
	}
	var errorSlice []error
	if directChildNode != nil {
		for k := 0; k < len(directChildNode.Content); k++ {
			err := checkDirectRuleReference(ruleSet, directChildNode.Content[k], ruleNameSlice)
			errorSlice = append(errorSlice, err)
		}
	}
	return joinError(errorSlice...)
}

func checkRuleSet(ruleSet map[string]tRule) error {
	var errorSlice []error
	mainRule, mainRuleFound := ruleSet["main"]
	if !mainRuleFound {
		err := fmt.Errorf("could not find the 'main' rule")
		errorSlice = append(errorSlice, err)
	} else {
		mainRule.isUsed = true
		ruleSet["main"] = mainRule
	}
	for name, rule := range ruleSet {
		if !rule.isUsed {
			err := fmt.Errorf("rule '%s' is defined but never used", name)
			errorSlice = append(errorSlice, err)
		}
	}
	for name, rule := range ruleSet {
		err := checkDirectRuleReference(ruleSet, rule.node, []string{name})
		errorSlice = append(errorSlice, err)
	}
	return joinError(errorSlice...)
}
