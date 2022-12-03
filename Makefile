SHELL := /bin/bash
NAME := http-file-server
CONTAINER_NAME := i0nw/${NAME}

REV := $(shell git rev-parse --short HEAD 2> /dev/null || echo 'unknown')

BRANCH     := $(shell git rev-parse --abbrev-ref HEAD 2> /dev/null  || echo 'unknown')
BUILD_DATE := $(shell date +%Y%m%d-%H:%M:%S)
BUILD_USER := $(shell whoami)

all: build

check: fmt build test

version:
ifeq (,$(wildcard pkg/version/VERSION))
TAG := $(shell git fetch --all -q 2>/dev/null && git describe --abbrev=0 --tags 2>/dev/null)
ON_EXACT_TAG := $(shell git name-rev --name-only --tags --no-undefined HEAD 2>/dev/null | sed -n 's/^\([^^~]\{1,\}\)\(\^0\)\{0,1\}$$/\1/p')
VERSION := $(shell [ -z "$(ON_EXACT_TAG)" ] && echo "$(TAG)-dev-$(REV)" | sed 's/^v//' || echo "$(TAG)" | sed 's/^v//' )
else
VERSION := $(shell cat pkg/version/VERSION)
endif

DOCKER_NETWORK := $(shell docker network ls --filter name=${NAME} -q)

print-version: version
	@echo $(VERSION)

print-rev:
	@echo $(REV)

print-branch:
	@echo $(BRANCH)

print-build-date:
	@echo $(BUILD_DATE)

print-build-user:
	@echo $(BUILD_USER)

build: version
	VERSION=${VERSION} REV=${REV} BRANCH=${BRANCH} BUILD_USER=${BUILD_USER} RUST_VERSION="$(shell rustc --version)" cargo build

build-release: version
	VERSION=${VERSION} REV=${REV} BRANCH=${BRANCH} BUILD_USER=${BUILD_USER} RUST_VERSION="$(shell rustc --version)" cargo build --release

docker-create-network:
ifeq ($(strip $(DOCKER_NETWORK)),)
	@echo Creating docker network ${NAME}...
	docker network create ${NAME}
else
	@echo Docker network ${NAME} already created.
endif

docker-build: print-version print-rev print-branch
	docker build . --build-arg DOCKER_ARG_VERSION=$(VERSION) --build-arg DOCKER_ARG_REV=$(REV) --build-arg DOCKER_ARG_BRANCH=$(BRANCH) --build-arg DOCKER_ARG_BUILD_USER=${BUILD_USER} -t ${CONTAINER_NAME}:latest
	docker tag ${CONTAINER_NAME}:latest ${CONTAINER_NAME}:$(VERSION)

docker-run: docker-create-network
	docker run --name ${NAME} --rm --network ${NAME} -p 8000:8000 ${CONTAINER_NAME}:latest

docker-run-d: docker-create-network docker-build
	docker run --name ${NAME} --rm -d --network ${NAME} -p 8000:8000 ${CONTAINER_NAME}:latest

docker-push:
	docker push ${CONTAINER_NAME}:latest
	docker push ${CONTAINER_NAME}:$(VERSION)

docker-test-functional:
	docker run --rm --network ${NAME} -e APP_URL=${NAME}:8000 -v $(shell pwd)/test/functional:/opt/bin grafana/k6 run /opt/bin/k6.js

kubernetes-rolling-update-current-version:
	kubectl set image -f kube/deployment.yaml app=${CONTAINER_NAME}:${VERSION}

kubernetes-rolling-update-latest:
	kubectl set image -f kube/deployment.yaml app=${CONTAINER_NAME}:latest

deploy: clean docker-build docker-push kubernetes-rolling-update-current-version

run:
	VERSION=${VERSION} REV=${REV} BRANCH=${BRANCH} BUILD_USER=${BUILD_USER} RUST_VERSION="$(shell rustc --version)" cargo run

test:
	cargo test

test-functional:
	APP_URL=0.0.0.0:8000 k6 run ./test/functional/k6.js

clean:
	rm -rf target

# This will stop make linking directories with these names to make commands
.PHONY: all test clean
