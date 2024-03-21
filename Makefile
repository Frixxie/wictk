PROJECT_NAME=wictk

all: container

build:
	cargo check --verbose
	cargo b --verbose

test: build
	cargo t --verbose

container: test
	docker build -t ghcr.io/frixxie/$(PROJECT_NAME):latest .

docker_login:
	docker login ghcr.io -u Frixxie -p $(GITHUB_TOKEN)

publish_container: container docker_login
	docker push ghcr.io/frixxie/$(PROJECT_NAME):latest

publish_tagged_container: container docker_login
	docker tag ghcr.io/frixxie/$(PROJECT_NAME):latest ghcr.io/frixxie/$(PROJECT_NAME):$(DOCKERTAG)
	docker push ghcr.io/frixxie/$(PROJECT_NAME):$(DOCKERTAG)
