package lidy

import "fmt"

func makeMetaParserFor(parser Parser) Parser {
	schema := YamlFile{
		File: ReadLocalFile("../../lidy.schema.yaml"),
	}
	err := schema.Unmarshal()
	if err != nil {
		panic(err)
	}

	builderMap := map[string]Builder{
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
			_, _mergeFound := mapData.Map["_merge"]
			if !_mapFound && !_mapFacultativeFound && !_mapOfFound && !_mergeFound {
				return nil, true, fmt.Errorf("expression maps must contain at least one mapChecker or listChecker keyword")
			}
			return input.data, true, nil
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

	ruleSet := makeRuleSet(schema, builderMap)

	return Parser(ruleSet)
}
