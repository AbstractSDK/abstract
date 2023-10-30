In order to run the tests, you need to follow those steps : 


1. Clone the following repo [git@github.com:AbstractSDK/interchaintest.git](git@github.com:AbstractSDK/interchaintest.git) and run

```sh
	go test examples/ibc/cw_ibc_test.go
``` 

2. Setup the interchain environement (long process) using
```sh
 cargo run --bin setup
```

3. Run the test 
```sh
 cargo test
```

If you want to avoid reuploading all the abstract core every time and simply reupload and link the client and host contracts, use
```sh
	cargo run --bin setup -- true
```