package main

import (
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"path/filepath"

	"golang.org/x/mod/modfile"
)

func main() {
	// Get the absolute path of this binary's directory
	dir, err := filepath.Abs(filepath.Dir(os.Args[0]))

	if err != nil {
		log.Fatal(err)
	}

	osmosisPath := filepath.Join(dir, "../../dependencies/osmosis/go.mod")
	libosmosistestingModPath := filepath.Join(dir, "../../packages/osmosis-testing/libosmosistesting/go.mod")

	osmosisMod := readMod(osmosisPath)
	libosmosistestingMod := readMod(libosmosistestingModPath)

	replaceModFileReplaceDirectives(osmosisMod, libosmosistestingMod)
	writeMod(libosmosistestingMod, libosmosistestingModPath)
}

func readMod(modPath string) *modfile.File {
	// Read the contents of the go.mod file
	bytes, err := ioutil.ReadFile(modPath)
	if err != nil {
		log.Fatal(err)
	}

	// Parse the go.mod file
	f, err := modfile.Parse(modPath, bytes, nil)
	if err != nil {
		log.Fatal(err)
	}

	return f
}

func replaceModFileReplaceDirectives(from, to *modfile.File) {
	fmt.Printf("Drop replace directives for `%s`:\n", to.Module.Mod.Path)

	// Drop all replace directives from `to` go.mod
	for _, rep := range to.Replace {
		fmt.Printf("  - %s %s => %s %s\n", rep.Old.Path, rep.Old.Version, rep.New.Path, rep.New.Version)
		to.DropReplace(rep.Old.Path, rep.Old.Version)
	}

	// Cleanup the go.mod file
	to.Cleanup()

	fmt.Println("---")

	fmt.Printf("Add replace directives for `%s`:\n", to.Module.Mod.Path)

	// Add all replace directives from `from` go.mod
	for _, rep := range from.Replace {
		fmt.Printf("  - %s %s => %s %s\n", rep.Old.Path, rep.Old.Version, rep.New.Path, rep.New.Version)
		to.AddReplace(rep.Old.Path, rep.Old.Version, rep.New.Path, rep.New.Version)
	}

	// Sort the blocks
	to.SortBlocks()
}

func writeMod(mod *modfile.File, modPath string) {
	// Write the go.mod file
	content, err := mod.Format()
	if err != nil {
		log.Fatal(err)
	}

	err = ioutil.WriteFile(modPath, content, 0644)
	if err != nil {
		log.Fatal(err)
	}
}
