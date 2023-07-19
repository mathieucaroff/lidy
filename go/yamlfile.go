package lidy

import yaml "gopkg.in/yaml.v3"

type YamlFile struct {
	File
	Yaml yaml.Node
}

func (yf *YamlFile) Unmarshal() error {
	return yaml.Unmarshal(yf.File.Content, yf.Yaml)
}
