package lidy

import (
	"fmt"
	"strconv"

	yaml "gopkg.in/yaml.v3"
)

func applySizeCheck(_min *yaml.Node, _max *yaml.Node, _nb *yaml.Node, content *yaml.Node) error {
	size := len(content.Content)
	if content.Kind == yaml.MappingNode {
		size /= 2
	}
	if _min != nil {
		min, _ := strconv.Atoi(_min.Value)
		if size < min {
			return checkError("_min", fmt.Sprintf("Expected container to have at least %d entries but it has only %d.", min, size), content)
		}
	}
	if _max != nil {
		max, _ := strconv.Atoi(_max.Value)
		if size > max {
			return checkError("_max", fmt.Sprintf("Expected container to have at most %d entries but it has %d.", max, size), content)
		}
	}
	if _nb != nil {
		nb, _ := strconv.Atoi(_nb.Value)
		if size != nb {
			return checkError("_nb", fmt.Sprintf("Expected container to have exactly %d entries but it has %d.", nb, size), content)
		}
	}

	return nil
}
