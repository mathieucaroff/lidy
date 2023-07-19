package lidy

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
	hasBeenBuilt bool
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
