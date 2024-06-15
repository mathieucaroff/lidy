package lidy

import "os"

type File struct {
	Name    string
	Content []byte
}

// ReadLocalFile reads a file from the file system and returns a lidy.File
func ReadLocalFile(path string) File {
	content, err := os.ReadFile(path)
	if err != nil {
		panic(err)
	}

	return File{
		Name:    path,
		Content: content,
	}
}
