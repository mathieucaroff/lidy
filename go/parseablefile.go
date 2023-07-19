package lidy

type Parseable struct {
	YamlFile
	err  error
	done bool
}

func (pa *Parseable) Parse() error {
	if pa.done {
		return pa.err
	}
	pa.err = pa.YamlFile.Unmarshal()
	if pa.err != nil {
		pa.done = true
		return pa.err
	}

	parser := Parser{Schema: LidyMetaSchema, Content: pa.YamlFile}
	pa.err = parser.Run()
	pa.done = true
	return pa.err
}
