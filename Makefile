PROJECT_NAME=wictk

all: test

build:
	cargo check --verbose
	cargo b --verbose

test: build
	cargo t --verbose

docker_builder:
	docker buildx create --name builder --platform linux/amd64,linux/arm64

docker_login:
	docker login ghcr.io -u Frixxie -p $(GITHUB_TOKEN)

container: docker_builder docker_login
	docker buildx build -t ghcr.io/frixxie/$(PROJECT_NAME):latest . --build-arg APIKEY=$(OPENWEATHERMAPAPIKEY) --platform linux/amd64,linux/arm64 --builder builder --push

container_tagged: docker_builder docker_login
	docker buildx build -t ghcr.io/frixxie/$(PROJECT_NAME):$(DOCKERTAG) . --build-arg APIKEY=$(OPENWEATHERMAPAPIKEY) --platform linux/amd64,linux/arm64 --builder builder --push
