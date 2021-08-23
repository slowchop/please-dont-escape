web:
	rustup target add wasm32-unknown-unknown
	cargo install wasm-pack
	wasm-pack build --target web --release
	mkdir dist/
	cp pkg/* dist/
	cp -r assets dist/