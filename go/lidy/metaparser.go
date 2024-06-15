package lidy

var metaParser = (func() Parser {
	schema := YamlFile{
		File: ReadLocalFile("../../lidy.schema.yaml"),
	}
	err := schema.Unmarshal()
	if err != nil {
		panic(err)
	}
	return Parser{
		schema: schema,
	}
})()
