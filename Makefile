build:
	docker compose build

start:
	docker compose up -d
	docker compose logs -f

stop:
	docker compose down --volumes --remove-orphans

restart: stop start

bash:
	docker compose run --rm aiutante bash
