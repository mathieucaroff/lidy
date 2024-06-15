package lidy

import (
	"errors"
	"fmt"

	yaml "gopkg.in/yaml.v3"
)

func checkError(keyword string, description string, node *yaml.Node) error {
	return fmt.Errorf("%s: %s %d:%d", keyword, description, node.Line, node.Column)
}

func joinError(err ...error) error {
	var remaining []error
	for _, e := range err {
		if e != nil {
			remaining = append(remaining, e)
		}
	}
	if len(remaining) == 0 {
		return nil
	}
	return errors.Join(remaining...)
}
