CONTAINER_NAME = nextcloud-test
APP_ID := ironcalc
APP_NAME := IronCalc
APP_VERSION := 0.1.0
APP_SECRET := 12345
APP_PORT := 2620
APP_HOST := host.docker.internal
NEXTCLOUD_URL := http://localhost:2180
DAEMON := install_$(APP_ID)

define JSON_INFO
	{
		"id": "$(APP_ID)",
		"name": "$(APP_NAME)",
		"daemon_config_name": "$(DAEMON)",
		"version": "$(APP_VERSION)",
		"secret": "$(APP_SECRET)",
		"port": $(APP_PORT),
		"routes": [
			{
				"url": ".*",
				"verb": "GET, POST, PUT, DELETE",
				"access_level": 1,
				"headers_to_exclude": []
			}
		]
	}
endef
export JSON_INFO

.PHONY: lint
lint:
	cd server && cargo fmt -- --check
	cd server && cargo clippy --all-targets --all-features -- -W clippy::unwrap_used -W clippy::expect_used -W clippy::panic -D warnings
	cd webapp/app.ironcalc.com/frontend/ && npm install && npm run check

.PHONY: format
format:
	cd server && cargo fmt

.PHONY: clean
clean:
	cd server && cargo clean

.PHONY: register
register:
	docker exec -u www-data $(CONTAINER_NAME) php occ app_api:app:unregister $(APP_ID) --silent --force || true
	docker exec -u www-data $(CONTAINER_NAME) php occ app_api:daemon:unregister $(DAEMON) || true
	docker exec -u www-data $(CONTAINER_NAME) php occ app_api:daemon:register $(DAEMON) $(APP_NAME) manual-install http $(APP_HOST) $(NEXTCLOUD_URL) || true
	docker exec -u www-data $(CONTAINER_NAME) php occ app_api:app:register $(APP_ID) $(DAEMON) --json-info "$$JSON_INFO" --force-scopes --wait-finish
