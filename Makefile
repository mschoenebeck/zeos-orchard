all: ./pkg

# using target no-modules to make sure web workers work in all browsers:
# https://rustwasm.github.io/wasm-bindgen/examples/wasm-in-web-worker.html#building--compatibility
# see also: https://rustwasm.github.io/docs/wasm-pack/commands/build.html#target
./pkg:
#	wasm-pack build --target no-modules --debug
	RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals' wasm-pack build --target web . -- -Z build-std=panic_abort,std
#	wasm-pack build --target web

clean:
	rm -rf ./pkg

# test with index.html in browser
#run: ./pkg test.js
#	node test.js
