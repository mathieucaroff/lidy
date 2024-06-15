package lidy

import (
	"fmt"
	"strconv"

	yaml "gopkg.in/yaml.v3"
)

func applySizeCheck(_min *yaml.Node, _max *yaml.Node, _nb *yaml.Node, content *yaml.Node) error {
	count := len(content.Content)
	if content.Kind == yaml.MappingNode {
		count /= 2
	}
	if _min != nil {
		min, _ := strconv.Atoi(_min.Value)
		if count < min {
			return checkError("_min", fmt.Sprintf("Expected container to have at least %d entries but it has only %d.", min, count), content)
		}
	}
	if _max != nil {
		max, _ := strconv.Atoi(_max.Value)
		if count > max {
			return checkError("_max", fmt.Sprintf("Expected container to have at most %d entries but it has %d.", max, count), content)
		}
	}
	if _nb != nil {
		nb, _ := strconv.Atoi(_nb.Value)
		if count != nb {
			return checkError("_nb", fmt.Sprintf("Expected container to have exactly %d entries but it has %d.", nb, count), content)
		}
	}

	return nil
}
