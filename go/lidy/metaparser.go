package lidy

import "fmt"

var builderMap = map[string]Builder{
	"mapChecker": func(input Result) (interface{}, bool, error) {
		mapData := input.data.(MapData)
		_, _mapFound := mapData.Map["_map"]
		_, _mapFacultativeFound := mapData.Map["_mapFacultative"]
		_, _mapOfFound := mapData.Map["_mapOf"]
		_, _mergeFound := mapData.Map["_merge"]
		if !_mapFound && !_mapFacultativeFound && !_mapOfFound && !_mergeFound {
			return nil, true, fmt.Errorf("expression maps must contain at least one mapChecker or listChecker keyword")
		}
		return input, true, nil
	},
}

var metaParser = (func() Parser {
	schema := YamlFile{
		File: ReadLocalFile("../../lidy.schema.yaml"),
	}
	err := schema.Unmarshal()
	if err != nil {
		panic(err)
	}
	return Parser{
		schema:     schema,
		builderMap: builderMap,
	}
})()
