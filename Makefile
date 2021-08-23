web:
	rustup default stable
	rustup target add wasm32-unknown-unknown
	cargo install wasm-pack
	wasm-pack build --target web --release
	rm dist/*
	cp pkg/* dist/
	cp -r assets dist/

web_docker:
	docker build -t please_dont_escape .

