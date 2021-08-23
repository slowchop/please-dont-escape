.PHONY: web

web:
	rustup default stable
	rustup target add wasm32-unknown-unknown
	cargo install wasm-pack
#	wasm-pack build --target web --release
	rm -rf dist
	mkdir dist
	cp web/* dist/
	cp pkg/* dist/
	cp -r assets dist/

web_docker:
	docker build -t please_dont_escape .

