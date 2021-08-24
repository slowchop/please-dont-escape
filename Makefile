.PHONY: web

web:
	rustup default stable
	rustup target add wasm32-unknown-unknown
	cargo install wasm-pack
	rm -rf pkg
	wasm-pack build --target web --release
	rm -rf dist
	mkdir dist
	cp web/* dist/
	cp pkg/* dist/
	cp -r assets dist/

web_deploy: web
	rm -fr please-dont-escape-deploy
	git clone git@github.com:slowchop/please-dont-escape-deploy.git
	cd please-dont-escape-deploy && rm -fr dist && cp -pr ../dist .
	cd please-dont-escape-deploy && git add . && git commit -m"deploy" && git push
	rm -fr please-dont-escape-deploy

web_docker:
	docker build -t please_dont_escape .

