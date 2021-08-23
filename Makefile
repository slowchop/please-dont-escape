wasm:
	cargo install wasm-pack
	wasm-pack build --target web --release
	mkdir dist/
	cp pkg/* dist/
	cp -r assets dist/