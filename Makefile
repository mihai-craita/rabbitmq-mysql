start_containers :
	docker-compose up -d --build

stop_containers :
	docker-compose down -v

publish_million_messages:
	pip3 install --upgrade pika
	python3 bench.py
