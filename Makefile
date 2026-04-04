# ── CalendarSync Docker 部署 ────────────────────────────
# 用法:
#   make build          本地构建镜像
#   make buildx         跨平台构建 (linux/amd64) 到本地
#   make push           构建并推送镜像
#   make deploy         推送后远程部署
#
# 代理设置 (可选，仅在需要时导出):
#   export http_proxy=http://127.0.0.1:7890
#   export https_proxy=http://127.0.0.1:7890
#   export all_proxy=socks5://127.0.0.1:7890

REGISTRY   := registry.cn-hangzhou.aliyuncs.com
IMAGE      := $(REGISTRY)/xinbaojian/calendar
TAG       ?= latest
PLATFORM  := linux/amd64
BUILDER   := cal-builder

# 代理配置 (从环境变量读取，自动转换 127.0.0.1 为 host.docker.internal)
DOCKER_HTTP_PROXY  := $(or $(HTTP_PROXY),$(http_proxy))
DOCKER_HTTPS_PROXY := $(or $(HTTPS_PROXY),$(https_proxy))
DOCKER_ALL_PROXY   := $(or $(ALL_PROXY),$(all_proxy))

# ── 构建命令 ─────────────────────────────────────────────

## 本地构建
build:
	docker build -t $(IMAGE):$(TAG) .

## 确保 container driver builder 存在
ensure-builder:
	@docker buildx inspect $(BUILDER) >/dev/null 2>&1 && \
		docker buildx use $(BUILDER) || { \
		BUILDER_OPTS="--name $(BUILDER) --driver docker-container --use"; \
		if [ -n "$(DOCKER_HTTP_PROXY)" ]; then \
			PROXY_HOST=$$(echo "$(DOCKER_HTTP_PROXY)" | sed 's|http://[^/]*:\([0-9]*\)|\1|'); \
			BUILDER_OPTS="$$BUILDER_OPTS --driver-opt env.http_proxy=http://host.docker.internal:$$PROXY_HOST"; \
			BUILDER_OPTS="$$BUILDER_OPTS --driver-opt env.https_proxy=http://host.docker.internal:$$PROXY_HOST"; \
		fi; \
		docker buildx create $$BUILDER_OPTS; \
	}

## 跨平台构建并加载到本地 (使用本地缓存)
buildx:
	@docker buildx use default 2>/dev/null || true
	docker buildx build --platform $(PLATFORM) \
		-t $(IMAGE):$(TAG) \
		--cache-to type=local,dest=/tmp/buildkit-cache \
		--cache-from type=local,src=/tmp/buildkit-cache \
		--load .

## 构建并推送 (依赖缓存通过 Dockerfile mount cache 实现)
push: ensure-builder
	docker buildx build --platform $(PLATFORM) \
		-t $(IMAGE):$(TAG) \
		--push .

## 带版本号推送 (用法: make release VERSION=1.0.0)
release:
	$(MAKE) TAG=$(VERSION) push
	$(MAKE) TAG=latest push

# ── 部署命令 ─────────────────────────────────────────────

SSH_HOST  ?= root@your-server
DEPLOY_DIR ?= /opt/calendar-sync

## 远程部署 (推送镜像 + SSH 拉取重启)
deploy: push
	ssh $(SSH_HOST) "docker pull $(IMAGE):$(TAG)"
	ssh $(SSH_HOST) "cd $(DEPLOY_DIR) && docker compose up -d"

# ── 清理 ─────────────────────────────────────────────────

## 清理本地构建缓存
clean:
	cargo clean
	docker image prune -f
	rm -rf /tmp/buildkit-cache

## 删除本地镜像
rmi:
	docker rmi $(IMAGE):$(TAG) 2>/dev/null || true

## 重建 builder (代理变更后使用)
rebuild-builder:
	docker buildx rm $(BUILDER) 2>/dev/null || true
	@$(MAKE) ensure-builder

.PHONY: build buildx push release deploy clean rmi ensure-builder rebuild-builder
