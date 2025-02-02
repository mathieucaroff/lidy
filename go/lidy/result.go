package lidy

import yaml "gopkg.in/yaml.v3"

type Position struct {
	filename string
	// The beginning line of the position
	line int
	// The beginning column in the line of the position
	column int
	// The ending line of the position
	lineEnd int
	// The ending column of the position
	columnEnd int
}

type Result struct {
	Position
	ruleName     string
	isLidyData   bool
	data         interface{}
}

// MapData -- Lidy result of a MapChecker
type MapData struct {
	// Map -- the named, individually-typed properties specified in _map
	Map map[string]Result
	// MapOf -- the unnamed entries of the map
	MapOf []KeyValueData
}

// KeyValueData -- A lidy key-value pair, usually part of a MapData
type KeyValueData struct {
	Key   Result
	Value Result
}

// ListData -- A lidy yaml sequence result
type ListData struct {
	List   []Result
	ListOf []Result
}

func makeResult(parserData tParserData, content *yaml.Node, data interface{}) Result {
	ruleName := parserData.ruleTrace[len(parserData.ruleTrace)-1]

	return Result{
		Position: Position{
			filename:  parserData.contentFileName,
			line:      content.Line,
			column:    content.Column,
			lineEnd:   content.Line,
			columnEnd: content.Column + len(content.Value),
		},
		ruleName:     ruleName,
		isLidyData:   true,
		data:         data,
	}
}
