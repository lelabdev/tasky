module tasky

go 1.24.4

require gopkg.in/yaml.v3 v3.0.1

require (
	github.com/BurntSushi/toml v1.5.0
	github.com/urfave/cli/v2 v2.27.7
)

require (
	github.com/cpuguy83/go-md2man/v2 v2.0.7 // indirect
	github.com/russross/blackfriday/v2 v2.1.0 // indirect
	github.com/xrash/smetrics v0.0.0-20240521201337-686a1a2994c1 // indirect
)

replace github.com/micmonay/keybd_event => /dev/null
