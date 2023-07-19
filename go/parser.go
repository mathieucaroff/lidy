package lidy

import (
	"errors"

	yaml "gopkg.in/yaml.v3"
)

type Parser struct {
	Schema  Parseable
	Content Parseable
	result  Result
	err     error
	done    bool
}

// Parse -- use the parser to check the given YAML file, and produce a Lidy Result.
func (p *Parser) Run() (Result, error) {
	if p.done {
		return p.result, p.err
	}
	ruleMap := map[string]*yaml.Node{}

	p.err = errors.Join(p.err, p.Schema.Parse(), p.Content.Parse())

	parserDocument := p.Schema.Yaml.Content[0]

	for k := range parserDocument.Content {
		ruleName := parserDocument.Content[k].Value
		ruleValue := parserDocument.Content[k+1]
		ruleMap[ruleName] = ruleValue
	}

	return result, err
}
