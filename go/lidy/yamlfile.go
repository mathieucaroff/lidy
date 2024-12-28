package lidy

import (
	"fmt"

	yaml "gopkg.in/yaml.v3"
)

type YamlFile struct {
	File
	Yaml         yaml.Node
	parsingError error
	doneParsing  bool
}

func (yf *YamlFile) Unmarshal() error {
	if yf.doneParsing {
		return yf.parsingError
	}
	yf.parsingError = yaml.Unmarshal(yf.File.Content, &yf.Yaml)
	if yf.parsingError == nil && len(yf.Yaml.Content) == 0 {
		yf.parsingError = fmt.Errorf("the document must contain at least one node")
	}
	yf.doneParsing = true
	return yf.parsingError
}
